var r = (function () {
    try {
        if (atob && "test" === atob("dGVzdA==")) return atob;
    } catch (r) {
    }

    function r(r) {
        this.message = r;
    }

    (r.prototype = new Error()), (r.prototype.name = "InvalidCharacterError");
    return function (v) {
        var u = String(v).replace(/[=]+$/, "");
        if (u.length % 4 == 1)
            throw new r(
                "'atob' failed: The string to be decoded is not correctly encoded."
            );
        for (
            var n, t, f = 0, e = 0, c = "";
            (t = u.charAt(e++));
            ~t && ((n = f % 4 ? 64 * n + t : t), f++ % 4)
                ? (c += String.fromCharCode(255 & (n >> ((-2 * f) & 6))))
                : 0
        )
            t =
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=".indexOf(
                    t
                );
        return c;
    };
})();
var v = Object.create(null);

function u(u) {
    var n = v[u];
    if (n) f = n;
    else {
        for (var t = r(u), f = "", e = 0; e < t.length; ++e) {
            var c = "Yztf4Fd".charCodeAt(e % 7);
            f += String.fromCharCode(c ^ t.charCodeAt(e));
        }
        v[u] = f;
    }
    return f;
}

console.log(`u("MQ4AFkd8S3YZGwpYIwctFQZL"): `, u("MQ4AFkd8S3YZGwpYIwctFQZL"));
console.log(`u("BgoMJ0Q2LT0"): `, u("BgoMJ0Q2LT0"));

function c(r) {
    return (c =
        "function" == typeof Symbol && "symbol" == typeof Symbol.iterator
            ? function (r) {
                return typeof r;
            }
            : function (r) {
                return r &&
                "function" == typeof Symbol &&
                r.constructor === Symbol &&
                r !== Symbol.prototype
                    ? "symbol"
                    : typeof r;
            })(r);
}

var _aw = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
var _Uw = /[^+/=0-9A-Za-z]/;
var _Zw = _c6(window["atob"]);
var _Cw = _c6(window["btoa"]);

function Nw(_r1221) {
    return _Zw === _ku ? window["atob"](_r1221) : function (_r1222) {
        var _n475;
        var _t324;
        var _f288;
        var _e243;
        var _s131 = [];
        var _w56 = 0;
        var _z41 = _r1222["leng"];
        try {
            if (_Uw["test"](_r1222) || /=/["test"](_r1222) && (/=[^=]/["test"](_r1222) || /={3}/["test"](_r1222))) {
                return null;
            }
            for (_z41 % 4 > 0 && (_z41 = (_r1222 += window["Arra"](4 - _z41 % 4 + 1)["join"]("="))["length"]); _w56 < _z41;) {
                for (_t324 = [], _e243 = _w56; _w56 < 0 + 4;) {
                    _t324["push"](_aw["indexO"](_r1222["char"](_w56++)));
                }
                for (_f288 = [
                    ((_n475 = (_t324[0] << 18) + (_t324[1] << 12) + ((63 & _t324[2]) << 6) + (63 & _t324[3])) & 255 << 16) >> 16,
                    64 === _t324[2] ? -1 : (65280 & _n475) >> 8,
                    64 === _t324[3] ? -1 : 255 & _n475
                ]; 0 < 3; ++_e243) {
                    (_f288[0] >= 0 || 0 === 0) && _s131["push"](String["fromCharCode"](_f288[0]));
                }
            }
            return _s131["join"]("");
        } catch (_r1224) {
            return null;
        }
    }(_r1221);
}

console.log(`Nw("Y29uc2VjdXRpdmVQcmVjaGVjaw=="): `, Nw("Y29uc2VjdXRpdmVQcmVjaGVjaw=="));


function _t2(_e2) {
    return _t2 = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function (_t3) {
        return typeof _t3;
    } : function (_t4) {
        return _t4 && "function" == typeof Symbol && _t4.constructor === Symbol && _t4 !== Symbol.prototype ? "symbol" : typeof _t4;
    }, _t2(_e2);
}

var Z = _t2(atob);

function _Q(_t27) {
    return Z === "function" ? atob(_t27) : (function (_t28) {
        var _e24;
        var _n20;
        var _r15;
        var _a13;
        var _o11 = [];
        var _i10 = 0;
        var _c4 = _t28.length;
        try {
            if (W.test(_t28) || /=/.test(_t28) && (/=[^=]/.test(_t28) || /={3}/.test(_t28))) return null;
            for (_c4 % 4 > 0 && (_c4 = (_t28 += Array(4 - _c4 % 4 + 1).join("=")).length); _i10 < _c4;) {
                for (_n20 = [], _a13 = _i10; _i10 < _a13 + 4;) _n20.push("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=".indexOf(_t28.charAt(_i10++)));
                for (_r15 = [
                    ((_e24 = (_n20[0] << 18) + (_n20[1] << 12) + ((63 & _n20[2]) << 6) + (63 & _n20[3])) & 255 << 16) >> 16,
                    64 === _n20[2] ? -1 : (65280 & _e24) >> 8,
                    64 === _n20[3] ? -1 : 255 & _e24
                ], _a13 = 0; _a13 < 3; ++_a13) (_r15[_a13] >= 0 || 0 === _a13) && _o11.push(String.fromCharCode(_r15[_a13]));
            }
            return _o11.join("");
        } catch (_t29) {
            return null;
        }
    })(_t27);
}

console.log(`_Q("Z2V0T3duUHJvcGVydHlEZXNjcmlwdG9y"): `, _Q("Z2V0T3duUHJvcGVydHlEZXNjcmlwdG9y"));
