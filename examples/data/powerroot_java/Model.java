public class Library {
    static int sqrtuf(int i);

    public static model int sqrt(int i){
        int rv = sqrtuf(i);
        if(i==0){
            assert rv == 0;
        }
        
        assert rv*rv <= i;
        assert (rv+1)*(rv+1) > i;
        return rv;
    }
}

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
        int k=2;
        if(k==0 || x==0) return;
        int val = PowerRoot.powerroot(k, x);
        assert(val !=0);
        assert(val == twokroot(x,k));
    }
}
