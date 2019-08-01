# cannyls_bencher

# ワークロード記述
```
# 乱数のシード
Seed: 42; 

# 100iterationで
# 上から順番に実行
Ordered[100] {
  <10%> Get;
  <20%> OverWrite(43);
  <30%> Delete;
  <39%> Delete(10, 20);
  <1%> DeleteRange(99, 100);
}

# 200iterationで
# 順不同実行
Unordered[200] {
  <10%> Get;
  <20%> OverWrite(43);
  <30%> Delete;
  <39%> Delete(10, 20);
  <1%> DeleteRange(99, 100);
}
```
