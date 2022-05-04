public class Stack {
    int[] storage;
    int pos;
    public Stack() {
        storage = new int[20];
        pos = 0;
    }
    public void push(int i) {
        if (pos >= 20) {
            return;
        }
        storage[pos] = i;
        pos = pos + 1;
    }
    public int pop() {
        if (pos == 0) {
            return 0;
        }
        pos = pos - 1;
        return storage[pos];
    }
}

public class Test {
    public static void test_stack(Stack st, int p, int s, int offset) {
        int adder = ??;
        int factor = ??;
        for (int i = 0; i < p; i++) {
            st.push(i * (s % factor + adder) + offset);
        }
    }
}
public class Main {
    harness public static void main(int p, int s, int offset) {
        assume p <= 3;
        Stack st = new Stack();

        Test.test_stack(st, p, s, offset);

        int last_pop = st.pop();
        for (int i = 0; i < p - 1; i++) {
            int new_pop = st.pop();
            assert new_pop + (s % 2 + 2) == last_pop;
            last_pop = new_pop;
        }
    }
}