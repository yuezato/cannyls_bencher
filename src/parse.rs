extern crate combine;

use combine::combinator::{attempt, skip_until};
use combine::error::ParseError;
use combine::parser::char::{digit, spaces, string};
use combine::{eof, many1, one_of, optional, sep_by, sep_end_by, token, Parser, Stream};

use super::*;

pub fn parse_line_comment<I>() -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    token('#').with(skip_until((token('\n').map(|_| ())).or(eof())))
}

pub fn parse_seed<I>() -> impl Parser<Input = I, Output = u64>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (string("Seed:").skip(spaces()), parse_num().skip(token(';'))).map(|(_, n)| n)
}

pub fn parse_workload<I>() -> impl Parser<Input = I, Output = Workload>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        spaces().with(attempt(optional(parse_seed()))),
        many1(parse_section()),
    )
        .map(|(seed, sections)| Workload { seed, sections })
}

fn parse_iter<I>() -> impl Parser<Input = I, Output = usize>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (token('['), parse_num(), token(']')).map(|(_, iter, _)| iter)
}

pub fn spaces_with_comments<I>() -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    sep_by(spaces(), parse_line_comment())
}

fn parse_section<I>() -> impl Parser<Input = I, Output = Section>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    spaces_with_comments()
        .with(parse_ordered().or(parse_unordered()))
        .skip(spaces_with_comments())
}

fn parse_ordered<I>() -> impl Parser<Input = I, Output = Section>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        spaces().with(string("Ordered")),
        parse_iter(),
        spaces().with(token('{')).skip(spaces()),
        sep_end_by(parse_freq_command(), spaces()),
        token('}').skip(spaces()),
    )
        .map(|(_, iter, _, commands, _)| Section {
            iter,
            inner: SectionInner::Ordered(commands),
        })
}

fn parse_unordered<I>() -> impl Parser<Input = I, Output = Section>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        spaces().with(string("Unordered")),
        parse_iter(),
        spaces().with(token('{')).skip(spaces()),
        sep_end_by(parse_freq_command(), spaces()),
        token('}').skip(spaces()),
    )
        .map(|(_, iter, _, commands, _)| Section {
            iter,
            inner: SectionInner::Unordered(commands),
        })
}

fn parse_freq_command<I>() -> impl Parser<Input = I, Output = (Freq, Command)>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        spaces().with(parse_freq()).skip(spaces()),
        parse_command().skip(token(';')),
    )
}

fn parse_command<I>() -> impl Parser<Input = I, Output = Command>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    newput()
        .or(overwrite())
        .or(attempt(get_with_perc()))
        .or(random_get())
        .or(attempt(delete_with_perc()))
        .or(attempt(delete_range()))
        .or(random_delete())
}

fn parse_freq<I>() -> impl Parser<Input = I, Output = Freq>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (token('<'), parse_num(), string("%>")).map(|(_, freq, _)| freq)
}

fn parse_num<U, I>() -> impl Parser<Input = I, Output = U>
where
    U: std::str::FromStr,
    U::Err: std::fmt::Debug,
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many1(digit()).map(|string: String| string.parse::<U>().unwrap())
}

pub fn parse_bytes_with_suffix<I>() -> impl Parser<Input = I, Output = Bytes>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (many1(digit()), one_of("KMG".chars())).map(|(string, suffix): (String, char)| {
        let num = string.parse::<usize>().unwrap();
        match suffix {
            'K' => num * 1024,
            'M' => num * 1024 * 1024,
            'G' => num * 1024 * 1024 * 1024,
            _ => unreachable!("bug"),
        }
    })
}

fn parse_bytes<I>() -> impl Parser<Input = I, Output = Bytes>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    attempt(parse_bytes_with_suffix()).or(parse_num())
}

fn newput<I>() -> impl Parser<Input = I, Output = Command>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (string("New"), token('('), parse_bytes(), token(')'))
        .map(|(_, _, num, _)| Command::NewPut(num))
}

fn overwrite<I>() -> impl Parser<Input = I, Output = Command>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (string("OverWrite"), token('('), parse_bytes(), token(')'))
        .map(|(_, _, num, _)| Command::Overwrite(num))
}

fn random_get<I>() -> impl Parser<Input = I, Output = Command>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    string("Get").map(|_| Command::RandomGet)
}

fn random_delete<I>() -> impl Parser<Input = I, Output = Command>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    string("Delete").map(|_| Command::RandomDelete)
}

fn get_with_perc<I>() -> impl Parser<Input = I, Output = Command>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        string("Get"),
        token('('),
        parse_num(),
        spaces().and(token(',')).and(spaces()),
        parse_num(),
        token(')'),
    )
        .map(|(_, _, num1, _, num2, _)| Command::Get(num1, num2))
}

fn delete_with_perc<I>() -> impl Parser<Input = I, Output = Command>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        string("Delete"),
        token('('),
        parse_num(),
        spaces().and(token(',')).and(spaces()),
        parse_num(),
        token(')'),
    )
        .map(|(_, _, num1, _, num2, _)| Command::Delete(num1, num2))
}

fn delete_range<I>() -> impl Parser<Input = I, Output = Command>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        string("DeleteRange"),
        token('('),
        parse_num(),
        spaces().and(token(',')).and(spaces()),
        parse_num(),
        token(')'),
    )
        .map(|(_, _, num1, _, num2, _)| Command::DeleteRange(num1, num2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newput_works() {
        assert_eq!(newput().parse("New(12)"), Ok((Command::NewPut(12), "")));
        assert_eq!(
            newput().parse("New(12K)"),
            Ok((Command::NewPut(12 * 1024), ""))
        );
        assert_eq!(
            newput().parse("New(12M)"),
            Ok((Command::NewPut(12 * 1024 * 1024), ""))
        );
    }

    #[test]
    fn overwrite_works() {
        assert_eq!(
            overwrite().parse("OverWrite(42)"),
            Ok((Command::Overwrite(42), ""))
        );
        assert_eq!(
            overwrite().parse("OverWrite(42K)"),
            Ok((Command::Overwrite(42 * 1024), ""))
        );
        assert_eq!(
            overwrite().parse("OverWrite(42M)"),
            Ok((Command::Overwrite(42 * 1024 * 1024), ""))
        );
    }

    #[test]
    fn random_get_works() {
        assert_eq!(random_get().parse("Get"), Ok((Command::RandomGet, "")));
    }

    #[test]
    fn get_with_perc_works() {
        assert_eq!(
            get_with_perc().parse("Get(12, 42)"),
            Ok((Command::Get(12, 42), ""))
        );
    }

    #[test]
    fn parse_freq_works() {
        assert_eq!(parse_freq().parse("<42%>"), Ok((42, "")));
    }

    #[test]
    fn delete_range_works() {
        assert_eq!(
            delete_range().parse("DeleteRange(42, 84)"),
            Ok((Command::DeleteRange(42, 84), ""))
        );
    }

    #[test]
    fn random_delete_works() {
        assert_eq!(
            random_delete().parse("Delete"),
            Ok((Command::RandomDelete, ""))
        );
    }

    #[test]
    fn parse_command_works() {
        assert_eq!(
            parse_command().parse("New(42)"),
            Ok((Command::NewPut(42), ""))
        );
        assert_eq!(
            parse_command().parse("OverWrite(42)"),
            Ok((Command::Overwrite(42), ""))
        );
        assert_eq!(parse_command().parse("Get"), Ok((Command::RandomGet, "")));
        assert_eq!(
            parse_command().parse("Get(42, 84)"),
            Ok((Command::Get(42, 84), ""))
        );
        assert_eq!(
            parse_command().parse("Delete"),
            Ok((Command::RandomDelete, ""))
        );
        assert_eq!(
            parse_command().parse("Delete(42, 84)"),
            Ok((Command::Delete(42, 84), ""))
        );
        assert_eq!(
            parse_command().parse("DeleteRange(42, 84)"),
            Ok((Command::DeleteRange(42, 84), ""))
        );
    }

    #[test]
    fn parse_freq_command_works() {
        assert_eq!(
            parse_freq_command().parse("<10%> New(42);"),
            Ok(((10, Command::NewPut(42)), ""))
        );
        assert_eq!(
            parse_freq_command().parse("<11%> OverWrite(42);"),
            Ok(((11, Command::Overwrite(42)), ""))
        );
        assert_eq!(
            parse_freq_command().parse("<12%> Get;"),
            Ok(((12, Command::RandomGet), ""))
        );
        assert_eq!(
            parse_freq_command().parse("<13%> Get(42, 84);"),
            Ok(((13, Command::Get(42, 84)), ""))
        );
        assert_eq!(
            parse_freq_command().parse("<14%> Delete;"),
            Ok(((14, Command::RandomDelete), ""))
        );
        assert_eq!(
            parse_freq_command().parse("<15%> Delete(42, 84);"),
            Ok(((15, Command::Delete(42, 84)), ""))
        );
        assert_eq!(
            parse_freq_command().parse("<16%> DeleteRange(42, 84);"),
            Ok(((16, Command::DeleteRange(42, 84)), ""))
        );
    }

    #[test]
    fn parse_ordered_works() {
        let syntax = r#"
Ordered[100] {
  <10%> Get;
  <20%> OverWrite(43);
  <30%> Delete;
  <39%> Delete(10, 20);
  <1%> DeleteRange(99, 100);
}
"#;
        let expected = Section {
            iter: 100,
            inner: SectionInner::Ordered(vec![
                (10, Command::RandomGet),
                (20, Command::Overwrite(43)),
                (30, Command::RandomDelete),
                (39, Command::Delete(10, 20)),
                (1, Command::DeleteRange(99, 100)),
            ]),
        };

        assert_eq!(parse_ordered().parse(syntax), Ok((expected, "")));
    }

    #[test]
    fn parse_unordered_works() {
        let syntax = r#"
Unordered[100] {
  <10%> Get;
  <20%> OverWrite(43);
  <30%> Delete;
  <39%> Delete(10, 20);
  <1%> DeleteRange(99, 100);
}
"#;
        let expected = Section {
            iter: 100,
            inner: SectionInner::Unordered(vec![
                (10, Command::RandomGet),
                (20, Command::Overwrite(43)),
                (30, Command::RandomDelete),
                (39, Command::Delete(10, 20)),
                (1, Command::DeleteRange(99, 100)),
            ]),
        };

        assert_eq!(parse_unordered().parse(syntax), Ok((expected, "")));
    }

    #[test]
    fn parse_bytes_works() {
        assert_eq!(parse_bytes().parse("42"), Ok((42, "")));
        assert_eq!(parse_bytes().parse("42K"), Ok((42 * 1024, "")));
        assert_eq!(parse_bytes().parse("42M"), Ok((42 * 1024 * 1024, "")));
    }

    #[test]
    fn parse_workload_works() {
        let workload = r#"
Ordered[100] {
  <10%> Get;
  <20%> OverWrite(43);
  <30%> Delete;
  <39%> Delete(10, 20);
  <1%> DeleteRange(99, 100);
}

Unordered[100] {
  <10%> Get;
  <20%> OverWrite(43);
  <30%> Delete;
  <39%> Delete(10, 20);
  <1%> DeleteRange(99, 100);
}
"#;

        let expected1 = Section {
            iter: 100,
            inner: SectionInner::Ordered(vec![
                (10, Command::RandomGet),
                (20, Command::Overwrite(43)),
                (30, Command::RandomDelete),
                (39, Command::Delete(10, 20)),
                (1, Command::DeleteRange(99, 100)),
            ]),
        };

        let expected2 = Section {
            iter: 100,
            inner: SectionInner::Unordered(vec![
                (10, Command::RandomGet),
                (20, Command::Overwrite(43)),
                (30, Command::RandomDelete),
                (39, Command::Delete(10, 20)),
                (1, Command::DeleteRange(99, 100)),
            ]),
        };

        let w = Workload {
            seed: None,
            sections: vec![expected1, expected2],
        };

        assert_eq!(parse_workload().parse(workload), Ok((w, "")));
    }

    #[test]
    fn parse_workload_with_seed_works() {
        let workload = r#"

Seed: 42;


Ordered[100] {
  <10%> Get;
  <20%> OverWrite(43);
  <30%> Delete;
  <39%> Delete(10, 20);
  <1%> DeleteRange(99, 100);
}

Unordered[100] {
  <10%> Get;
  <20%> OverWrite(43);
  <30%> Delete;
  <39%> Delete(10, 20);
  <1%> DeleteRange(99, 100);
}
"#;

        let expected1 = Section {
            iter: 100,
            inner: SectionInner::Ordered(vec![
                (10, Command::RandomGet),
                (20, Command::Overwrite(43)),
                (30, Command::RandomDelete),
                (39, Command::Delete(10, 20)),
                (1, Command::DeleteRange(99, 100)),
            ]),
        };

        let expected2 = Section {
            iter: 100,
            inner: SectionInner::Unordered(vec![
                (10, Command::RandomGet),
                (20, Command::Overwrite(43)),
                (30, Command::RandomDelete),
                (39, Command::Delete(10, 20)),
                (1, Command::DeleteRange(99, 100)),
            ]),
        };

        let w = Workload {
            seed: Some(42),
            sections: vec![expected1, expected2],
        };

        assert_eq!(parse_workload().parse(workload), Ok((w, "")));
    }

    #[test]
    fn parse_seed_works() {
        assert_eq!(
            parse_seed().parse("Seed: 1234567890;"),
            Ok((1234567890, ""))
        );
    }

    #[test]
    fn parse_line_comment_works() {
        assert_eq!(
            parse_line_comment().parse("# line comment   test"),
            Ok(((), ""))
        );
        assert_eq!(
            parse_line_comment().parse(
                r#"# line comment   test
"#
            ),
            Ok(((), "\n"))
        );
        assert_eq!(parse_line_comment().parse("# comment    "), Ok(((), "")));
    }

    #[test]
    fn spaces_with_comments_works() {
        assert_eq!(
            spaces_with_comments().parse("   #comment1   "),
            Ok(((), ""))
        );

        assert_eq!(
            spaces_with_comments().parse(
                r#"

#comment1

# comment2

    # comment3

"#
            ),
            Ok(((), ""))
        );
    }

    #[test]
    fn parse_section_works() {
        let syntax = r#"
# Comment for the following section:
# Iteration 100
# 10-times Get
# 20-times 43bytes New Put
# ...
Ordered[100] {
  <10%> Get;
  <20%> New(43);
  <30%> Delete;
  <39%> Delete(10, 20);
  <1%> DeleteRange(99, 100);
}
"#;
        let expected = Section {
            iter: 100,
            inner: SectionInner::Ordered(vec![
                (10, Command::RandomGet),
                (20, Command::NewPut(43)),
                (30, Command::RandomDelete),
                (39, Command::Delete(10, 20)),
                (1, Command::DeleteRange(99, 100)),
            ]),
        };

        let r = parse_section().easy_parse(syntax);

        assert_eq!(r, Ok((expected, "")));
    }
}
