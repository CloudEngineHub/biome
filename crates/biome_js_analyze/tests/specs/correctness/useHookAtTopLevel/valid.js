/* should not generate diagnostics */
/* does not generate diagnostics */

function Component1({ a }) {
    useEffect();
    const [name, setName] = useState("");
    const value = useContext();
    const memoizedCallback = useCallback();

    const otherValue = useValue() || defaultValue;

    {
        useEffect();
    }
}

const implicitReturn = (x) => useMemo(() => x, [x]);

const implicitReturnInsideWrappedComponent = forwardRef((x) => useMemo(() => x, [x]));

function useHookInsideObjectLiteral() {
    return {
        abc: useCallback(() => null, [])
    };
}

function useHookInsideArrayLiteral() {
    return [useCallback(() => null, [])];
}

function useHookInsideFinallyClause() {
    try {
    } finally {
        useCleanUp();
    }
}

function useHookInsideFinallyClause2() {
    try {
    } catch (error) {
    } finally {
        useCleanUp();
    }
}

function useHookToCalculateKey(key) {
    const object = {};
    object[useObjectKey(key)] = true;
    return object;
}

function useKeyOfHookResult(key) {
    return useContext(Context)[key];
}

function usePropertyOfHookResult() {
    return useContext(Context).someProp;
}

const obj = {
    Component() {
        const [count, setCount] = useState(0);

        return count;
    }
}

// Hook called indirectly
function helper() {
    useEffect();
}

function Component2({ a }) {
    helper();
}

const Component3 = () => {
    useEffect();
};

export function Component4() {
    useEffect();
};

export default function Component5() {
    useEffect();
};

const Component6 = () => {
    return useState();
};

const Component7 = () => {
    const value = useRef().value;
    const [_val, _setter] = useState(useMemo('hello'));
}

function Component8() {
    const a = () => {
        return;
    };

    useEffect();
};

test('a', () => {
    function TestComponent() {
        useState();
        useHook();
    }

    render(<TestComponent />);
});

test('b', () => {
    const TestComponent = () => {
        useState();
        useHook();
    };

    render(<TestComponent />);
});

test('c', () => {
    const { result } = renderHook(() => useHook());

    expect(result.current).toBeDefined();
});

describe("foo", () => {
  beforeEach(() => jest.useFakeTimers('legacy'));
  afterEach(() => jest.useRealTimers());
});

describe("bar", () => {
  beforeEach(() => vi.useFakeTimers('legacy'));
  afterEach(() => vi.useRealTimers());
});

class DemoMethod {
    useHook() {
        return useMemo(() => "string", []);
    }
}

class DemoProperty {
    useHook = () => {
        return useMemo(() => "string", []);
    }
}
