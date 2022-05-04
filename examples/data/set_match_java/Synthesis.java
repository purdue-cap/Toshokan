public class Set {
    static int[] size_hist = new int[] { {{expand-to-java-hist-arrays (subtree logs "Set.size()") n_unknowns hist_cap}} };
    static int[] size_hist_len = new int[] { {{expand-to-hist-lens (subtree logs "Set.size()") n_unknowns}} };
    static int[] size_rtn = new int[] { {{expand-to-rtn-array (subtree logs "Set.size()") "??" n_unknowns}} };

    static int[] contains_hist = new int[] { {{expand-to-java-hist-arrays (subtree logs "Set.contains(int)") n_unknowns hist_cap}} };
    static int[] contains_hist_len = new int[] { {{expand-to-hist-lens (subtree logs "Set.contains(int)") n_unknowns}} };
    static int[] contains_rtn = new int[] { {{expand-to-rtn-array (subtree logs "Set.contains(int)") "??" n_unknowns}} };

    int[] hist;
    int hist_len;
    public Set() {
        hist = new int[{{hist_cap}}];
        hist[0] = {{ subtree func_hist_codes "Set()"}};
        hist_len = 1;
    }
    public int size() {
        if (hist_len > 0) {
            hist[](1, hist_len) = hist[](0, hist_len);
        }
        hist[0] = {{ subtree func_hist_codes "Set.size()" }};
        hist_len = hist_len + 1;

        {{#for-cap-logs (subtree logs "Set.size()") n_unknowns}}
        if (size_hist_len[{{@index}}] == hist_len &&
            size_hist[]({{(mul hist_cap @index)}}, hist_len) == hist[](0, hist_len) ) {
            return size_rtn[{{@index}}];
        }
        {{/for-cap-logs}}
        assert false;
        return 0;
    }
    public int contains(int i) {
        if (hist_len > 0) {
            hist[](1, hist_len) = hist[](0, hist_len);
        }
        hist[0] = {{ subtree func_hist_codes "Set.contains(int)" }};
        hist[hist_len + 1] = i;
        hist_len = hist_len + 2;

        {{#for-cap-logs (subtree logs "Set.contains(int)") n_unknowns}}
        if (contains_hist_len[{{@index}}] == hist_len &&
            contains_hist[]({{(mul hist_cap @index)}}, hist_len) == hist[](0, hist_len) ) {
            return contains_rtn[{{@index}}];
        }
        {{/for-cap-logs}}
        assert false;
        return 0;
    }
    public void add(int i) {
        if (hist_len > 0) {
            hist[](1, hist_len) = hist[](0, hist_len);
        }
        hist[0] = {{ subtree func_hist_codes "Set.add(int)" }};
        hist[hist_len + 1] = i;
        hist_len = hist_len + 2;
        return;
    }
}

public class Main {
    harness public static void main(int p, int s, int offset) {
        {{expand-x-d-points-to-assume c_e_s "p" "s" "offset"}}
        assume p <= 3;
        Set st = new Set();

        Test.test_set(st, p, s, offset);

        int check_value = offset;
        for (int i = 0; i < p; i++) {
            int contains = st.contains(check_value);
            assert contains == 1;
            check_value =  check_value + s % 2 + 2;
        }
    }
}