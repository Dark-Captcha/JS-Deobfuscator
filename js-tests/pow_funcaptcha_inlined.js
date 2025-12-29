// Copy of pow_funcaptcha.js with local wrapper p0 inlined to G
// Original: const p0 = G, -> replaced all p0(...) with G(...)

function m() {
  const pe = [
    "iypWL",
    "apply",
    "random",
    "TlKVo",
    "HeStn",
    "lPWuZ",
    "ERROR",
    "wWdod",
    "ker",
    "work_timeo",
    "g proof of",
    "lwMeG",
    "slice",
    "1842352bOjTaf",
    "message",
    "OBtQn",
    "map",
    "reduce",
    "toString",
    "hashRate",
    "repeat",
    "lzFmC",
    "YILeA",
    "RiwCt",
    "FFDOQ",
    "UGJhL",
    "Vxmbg",
    "inside wor",
    "pHUpE",
    "runtime_er",
    "nGcEA",
    "result",
    "endTime",
    "IbXWY",
    "ternal",
    "charCodeAt",
    "RnUQe",
    "start",
    "stack",
    "MpTCK",
    "wqneV",
    "floor",
    "105MpJejP",
    "split",
    "sEbwG",
    "vnmrP",
    "difficulty",
    "DONE",
    "mZBYm",
    "dsvWD",
    "pxwYk",
    "Error when",
    "XPVMU",
    "MlCGg",
    "971874lBcRZX",
    "POW_TIMEOU",
    "now",
    "JHatJ",
    "UQsWD",
    "yrcgt",
    "zDzAL",
    "18373696LNOVsW",
    "ror",
    "forEach",
    "undefined",
    "error",
    "1358760RqfNzx",
    "round",
    "215982iVTNPq",
    "ohPGf",
    "No Valid M",
    "366959kRyhHx",
    "xShhn",
    " work",
    "usNlo",
    "rBmtt",
    "xxZgL",
    "itimeout",
    "join",
    "kjETw",
    "seed",
    "substr",
    "DmmMm",
    "AOYaZ",
    "fromCharCo",
    "length",
    "YMjQy",
    "kJDNQ",
    "Timed out ",
    "substring",
    "1351565FHTyIJ",
    "keys",
    "startTime",
    "stringify",
    " processin",
    "aSJCm",
    "plvVY",
    "twnYg",
    "data",
    "match",
    "loaded",
    "essage typ",
  ];
  m = function () {
    return pe;
  };
  return m();
}

function G(p, g) {
  const J = m();
  return (
    (G = function (z, Q) {
      z = z - 0xf6;
      let j = J[z];
      return j;
    }),
    G(p, g)
  );
}

// Shuffler IIFE - p0 inlined to G
(function (p, g) {
  const J = p();
  while (!![]) {
    try {
      const z =
        parseInt(G(0x13b)) / 0x1 +
        parseInt(G(0x12a)) / 0x2 +
        parseInt(G(0x136)) / 0x3 +
        parseInt(G(0x101)) / 0x4 +
        parseInt(G(0x14e)) / 0x5 +
        (parseInt(G(0x138)) / 0x6) * (parseInt(G(0x11e)) / 0x7) +
        -parseInt(G(0x131)) / 0x8;
      if (z === g) break;
      else J["push"](J["shift"]());
    } catch (Q) {
      J["push"](J["shift"]());
    }
  }
})(m, 0x44598);

// Test calls using G directly
console.log("G(0xf6):", G(0xf6));     // index 0 -> "round"
console.log("G(0xf7):", G(0xf7));     // index 1 -> "length"  
console.log("G(0x12b):", G(0x12b));   // "POW_TIMEOU"
console.log("G(0x135):", G(0x135));   // "error"
console.log("G(0x156):", G(0x156));   // "data"
