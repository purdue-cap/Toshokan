public class Library {
    static int[] aarr = new int[] { {{expand-to-arg-array (subtree logs "Library.gcd(int, int)") 0 "??" n_unknowns}} };
    static int[] barr = new int[] { {{expand-to-arg-array (subtree logs "Library.gcd(int, int)") 1 "??" n_unknowns}} };
    static int[] rarr = new int[] { {{expand-to-rtn-array (subtree logs "Library.gcd(int, int)") "??" n_unknowns}} };

    public static int gcd(int a, int b) {
        {{#for-cap-logs (subtree logs "Library.gcd(int, int)") n_unknowns}}
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
    harness public static void main(int x0, int x1, int x2, int x3, int x4) {
        {{expand-x-d-points-to-assume c_e_s "x0" "x1" "x2" "x3" "x4"}}
        int [] array = new int[] {x0, x1, x2, x3, x4};
        for (int i = 0; i < array.length; i++)
            if (array[i]== 0) return;
        if (array.length < 2) return;
        int result = GCD.gcd_n(array);

        for (int i = 0; i < array.length; i ++)
            assert array[i] % result == 0;
        
        for (int i = result + 1; i < array[0]; i++) {
            boolean divisible = true;
            for (int j = 0; j < array.length; j ++)
                divisible = divisible && (array[j] % i == 0);
            assert !divisible;
        }
    }
}
