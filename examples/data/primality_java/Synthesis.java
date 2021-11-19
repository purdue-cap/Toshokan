public class Library {
    static int[] iarr = new int[] { {{expand-to-arg-array (subtree logs "Library.sqrt(int)") 0 "??" n_unknowns}} };
    // iarr = {i0, i1, ..., in, ??, ??, ..., ??};
    static int[] rarr = new int[] { {{expand-to-rtn-array (subtree logs "Library.sqrt(int)") "??" n_unknowns}} };
    // rarr = {r0, r1, ..., rn, ??, ??, ..., ??};

    public static int sqrt(int i) {
        {{#for-cap-logs (subtree logs "Library.sqrt(int)") n_unknowns}}
        if (iarr[{{@index}}] == i) {
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
    static boolean primality_spec(int p){
        if(p<=1) return false;
        if(p==2) return true;
        for(int i=2;i<p;i++){
            if(p%i == 0) return false;
        }
        return true;
    }

    harness public static void main(int x) {
        {{expand-points-to-assume c_e_s.[0] "x"}}
        assert Primality.primality(x) == primality_spec(x);
    }
}
