public class ArrayListP {
    static int[] get_hist = new int[] { {{expand-to-java-hist-arrays (subtree logs "ArrayListP.get(int)") n_unknowns hist_cap "??(4)"}} };
    static int[] get_hist_len = new int[] { {{expand-to-hist-lens (subtree logs "ArrayListP.get(int)") n_unknowns "??(8)"}} };
    static int[] get_rtn = new int[] { {{expand-to-rtn-array (subtree logs "ArrayListP.get(int)") "??(8)" n_unknowns}} };

    int[] hist;
    int hist_len;
    public ArrayListP() {
        hist = new int[{{hist_cap}}];
        hist[0] = {{ subtree func_hist_codes "ArrayListP()"}};
        hist_len = 1;
    }
    public int get(int i) {
        if (hist_len > 0) {
            hist[](1, {{hist_cap}} - 1) = hist[](0, {{hist_cap}} - 1);
        }
        hist[0] = {{ subtree func_hist_codes "ArrayListP.get(int)" }};
        hist[hist_len + 1] = i;
        hist_len = hist_len + 2;

        {{#for-cap-logs (subtree logs "ArrayListP.get(int)") n_unknowns}}
        if (get_hist_len[{{@index}}] == hist_len &&
            get_hist[]({{(mul hist_cap @index)}}, {{hist_cap}}) == hist[](0, {{hist_cap}}) ) {
            return get_rtn[{{@index}}];
        }
        {{/for-cap-logs}}
        assert false;
        return 0;
    }
    public void add(int i) {
        if (hist_len > 0) {
            hist[](1, {{hist_cap}} - 1) = hist[](0, {{hist_cap}} - 1);
        }
        hist[0] = {{ subtree func_hist_codes "ArrayListP.add(int)" }};
        hist[hist_len + 1] = i;
        hist_len = hist_len + 2;
        return;
    }
    public void set(int i, int e) {
        if (hist_len > 0) {
            hist[](1, {{hist_cap}} - 1) = hist[](0, {{hist_cap}} - 1);
        }
        hist[0] = {{ subtree func_hist_codes "ArrayListP.set(int, int)" }};
        hist[hist_len + 1] = i;
        hist[hist_len + 2] = e;
        hist_len = hist_len + 3;
        return;
    }
}

public class Main {

    harness public static void main(int x, int y, int z) {
        {{expand-x-d-points-to-assume c_e_s "x" "y" "z"}}
    	assume x != y && x != z && y != z;
    	assume x > 0 && x < 10;
    	assume y > 0 && y < 10;
    	assume z > 0 && z < 10;

        ArrayListP al = new ArrayListP();
        al.add(x);
        al.add(y);
        al.add(z);

        BubbleSort.length = 3;
        BubbleSort.bubbleSort(al);

        for (int i = 0; i < BubbleSort.length - 1; i ++) {
            int left = al.get(i);
            int right = al.get(i+1);
            assert left <= right;
        }
    }
}

