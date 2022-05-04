public class ArrayListP {
    static int[] get_hist = new int[] { {{expand-to-java-hist-arrays (subtree logs "ArrayListP.get(int)") n_unknowns hist_cap}} };
    static int[] get_hist_len = new int[] { {{expand-to-hist-lens (subtree logs "ArrayListP.get(int)") n_unknowns}} };
    static int[] get_rtn = new int[] { {{expand-to-rtn-array (subtree logs "ArrayListP.get(int)") "??" n_unknowns}} };

    int[] hist;
    int hist_len;
    public ArrayListP() {
        hist = new int[{{hist_cap}}];
        hist[0] = {{ subtree func_hist_codes "ArrayListP()"}};
        hist_len = 1;
    }
    public int get(int i) {
        if (hist_len > 0) {
            hist[](1, hist_len) = hist[](0, hist_len);
        }
        hist[0] = {{ subtree func_hist_codes "ArrayListP.get(int)" }};
        hist[hist_len + 1] = i;
        hist_len = hist_len + 2;

        {{#for-cap-logs (subtree logs "ArrayListP.get(int)") n_unknowns}}
        if (get_hist_len[{{@index}}] == hist_len &&
            get_hist[]({{(mul hist_cap @index)}}, hist_len) == hist[](0, hist_len) ) {
            return get_rtn[{{@index}}];
        }
        {{/for-cap-logs}}
        assert false;
        return 0;
    }
    public void push_back(int i) {
        if (hist_len > 0) {
            hist[](1, hist_len) = hist[](0, hist_len);
        }
        hist[0] = {{ subtree func_hist_codes "ArrayListP.push_back(int)" }};
        hist[hist_len + 1] = i;
        hist_len = hist_len + 2;
        return;
    }
}

public class Main {
    harness public static void main(int p, int s, int offset) {
        {{expand-x-d-points-to-assume c_e_s "p" "s" "offset"}}
        assume p <= 3;
        ArrayListP list = new ArrayListP();

        Test.test_arraylist(list, p, s, offset);

        int n1 = list.get(0);
        for (int i = 1; i < p; i++) {
            int n2 = list.get(i);
            assert n2 - n1 == s % 2 + 2;
            n1 = n2;
        }
    }
}