@JavaCodeGen
public class PowerRoot {
    static boolean term_cond(int i, int k) {
        int factor = ??;
        int offset = ??;
        int symbol = ??%2;
        int comparison = ??%4;
        int rhs;
        if (symbol == 0) {
            rhs = factor * k + offset;
        } else {
            rhs = factor * k - offset;
        }
        if (comparison == 0) {
            return i < rhs;
        }
        if (comparison == 1) {
            return i > rhs;
        }
        if (comparison == 2) {
            return i <= rhs;
        }
        return i >= rhs;
    }

    static int start_i() {
        int i = ??;
        return i;
    }

    public static int powerroot(int k, int x) {
        int val = x;
        for(int i=start_i();term_cond(i, k);i++){
            if(val != 1 && val != 0){
                val = Library.sqrt(val);
            }
        }
        return val;
    }
}