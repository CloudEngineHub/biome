/* should not generate diagnostics */

const foo = 3;

function b() {
	const a = 10;
}

if (true) {
	const a = 20;
}

const c = function c() {}

const d = class d {}
