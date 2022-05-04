public class Stack {
    static int[] pop_hist = new int[] { {{expand-to-java-hist-arrays (subtree logs "Stack.pop()") n_unknowns hist_cap}} };
    static int[] pop_hist_len = new int[] { {{expand-to-hist-lens (subtree logs "Stack.pop()") n_unknowns}} };
    static int[] pop_rtn = new int[] { {{expand-to-rtn-array (subtree logs "Stack.pop()") "??" n_unknowns}} };

    int[] hist;
    int hist_len;
    public Stack() {
        hist = new int[{{hist_cap}}];
        hist[0] = {{ subtree func_hist_codes "Stack()"}};
        hist_len = 1;
    }
    public void push(int i) {
        if (hist_len > 0) {
            hist[](1, hist_len) = hist[](0, hist_len);
        }
        hist[0] = {{ subtree func_hist_codes "Stack.push(int)" }};
        hist[hist_len + 1] = i;
        hist_len = hist_len + 2;
        return;
    }
    public int pop() {
        if (hist_len > 0) {
            hist[](1, hist_len) = hist[](0, hist_len);
        }
        hist[0] = {{ subtree func_hist_codes "Stack.pop()" }};
        hist_len = hist_len + 1;

        {{#for-cap-logs (subtree logs "Stack.pop()") n_unknowns}}
        if (pop_hist_len[{{@index}}] == hist_len &&
            pop_hist[]({{(mul hist_cap @index)}}, hist_len) == hist[](0, hist_len) ) {
            return pop_rtn[{{@index}}];
        }
        {{/for-cap-logs}}
        assert false;
        return 0;
    }
}

public class Main {
    harness public static void main(int p, int s, int offset) {
        {{expand-x-d-points-to-assume c_e_s "p" "s" "offset"}}
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