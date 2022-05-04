public class Library {
   public static int sqrt(int num) {
        if (num==1 || num==0) return num;
        if (num<0) assert false;
        int low=0;
        int mid;
        int high=1+(num/2);
        while (low+1<high){
            mid=low+(high-low)/2;
            if (num %mid == 0 && num/mid == mid)
                return mid;
            else if (mid<=num/mid)
                low=mid;
            else
                high=mid;
        }
        return low;
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
