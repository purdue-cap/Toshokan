public class Heap {
    static int[] popMin_hist = new int[] { {{expand-to-java-hist-arrays (subtree logs "Heap.popMin()") n_unknowns hist_cap}} };
    static int[] popMin_hist_len = new int[] { {{expand-to-hist-lens (subtree logs "Heap.popMin()") n_unknowns "??(5)"}} };
    static int[] popMin_rtn = new int[] { {{expand-to-rtn-array (subtree logs "Heap.popMin()") "??(5)" n_unknowns}} };

    int[] hist;
    int hist_len;
    public Heap() {
        hist = new int[{{hist_cap}}];
        hist[0] = {{ subtree func_hist_codes "Heap()"}};
        hist_len = 1;
    }
    public int popMin() {
        if (hist_len > 0) {
            hist[](1, hist_len) = hist[](0, hist_len);
        }
        hist[0] = {{ subtree func_hist_codes "Heap.popMin()" }};
        hist_len = hist_len + 1;

        {{#for-cap-logs (subtree logs "Heap.popMin()") n_unknowns}}
        if (popMin_hist_len[{{@index}}] == hist_len &&
            popMin_hist[]({{(mul hist_cap @index)}}, hist_len) == hist[](0, hist_len) ) {
            return popMin_rtn[{{@index}}];
        }
        {{/for-cap-logs}}
        assert false;
        return 0;
    }
    public void insert(int i) {
        if (hist_len > 0) {
            hist[](1, hist_len) = hist[](0, hist_len);
        }
        hist[0] = {{ subtree func_hist_codes "Heap.insert(int)" }};
        hist[hist_len + 1] = i;
        hist_len = hist_len + 2;
        return;
    }
}

public class Main {
    harness public static void main(int i0, int i1, int i2) {
        {{expand-x-d-points-to-assume c_e_s "i0" "i1" "i2"}}
        int[] input = new int[]{i0, i1, i2};
        int[] output = HeapSort.heapsort(input);

        for (int i = 0; i < input.length; i++) {
            boolean present_in_output = false;
            for (int j = 0; j < input.length; j++) {
                if (output[j] == input[i]) present_in_output = true;
            }
            assert present_in_output;
        }
        for (int i = 0; i < input.length-1; i++) {
            assert output[i] <= output[i+1];
        }
    }
}