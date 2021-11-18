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
    static int twokroot(int num, int k) {
        if(num==0) return 0;
        if (num==1) return 1;
        for(int i=2;i<num;i++){
            int kpow=i;
            for(int j=0;j<k;j++){
                kpow = (kpow*kpow);
                if(kpow > num) return i-1;
            }
            
        }
    return 1;
    }

    harness public static void main(int x) {
        {{expand-points-to-assume c_e_s.[0] "x"}} // assume(x == 0 || x == 1)
        int k=2;
        if(k==0 || x==0) return;
        int val = PowerRoot.powerroot(k, x);
        assert(val !=0);
        assert(val == twokroot(x,k));
    }
}
