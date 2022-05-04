public class Library {
    static int[] aarr = new int[] { {{expand-to-arg-array (subtree logs "Library.pow(int, int)") 0 "??" n_unknowns}} };
    static int[] barr = new int[] { {{expand-to-arg-array (subtree logs "Library.pow(int, int)") 1 "??" n_unknowns}} };
    static int[] rarr = new int[] { {{expand-to-rtn-array (subtree logs "Library.pow(int, int)") "??" n_unknowns}} };

    public static int pow(int a, int b) {
        {{#for-cap-logs (subtree logs "Library.pow(int, int)") n_unknowns}}
        if (aarr[{{@index}}] == a && barr[{{@index}}] == b) {
            return rarr[{{@index}}];
        }
        {{/for-cap-logs}}
        /*
        if (iarr[0]) == i) {
            return rarr[0]
        }
        ...
         */
        assert false;
    }
}

public class Main {
    harness public static void main(int p_0, int p_1, int p_2, int p_3, int x) {
        {{expand-x-d-points-to-assume c_e_s "p_0" "p_1" "p_2" "p_3" "x"}}
        int[] p = new int[]{p_0, p_1, p_2, p_3};

        int num = EvalPoly.evalpoly(p, x);

        int ref_num = 0;
        for (int j=0; j < p.length; j++){
            ref_num += p[j] * Library.pow(x, j);
        }

        assert num == ref_num;
    }
}
