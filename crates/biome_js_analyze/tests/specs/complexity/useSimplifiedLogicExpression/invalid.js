const r = true && boolExp;
const boolExp2 = true;
const r2 = boolExp || true;
const nonNullExp = 123;
const r3 = null ?? nonNullExp;
const boolExpr1 = true;
const boolExpr2 = false;
const r4 = /*1*/ !boolExpr1 /*2*/  || /*3*/ !boolExpr2 /*4*/ 
