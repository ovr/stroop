function bench(lc, fc) {
  var n, fact;
  var res = 0;
  while (--lc >= 0) {
    n = fc;
    fact = n;
    while (--n > 1)
      fact *= n;
    res += fact;
  }
  return res;
}

var res = bench(4e6, 100);
console.log(res);
