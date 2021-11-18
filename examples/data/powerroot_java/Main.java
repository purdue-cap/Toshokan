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

    public static void main(int x) {
        int k=2;
        int val = PowerRoot.powerroot(k, x);
        assert(val !=0);
        assert(val == twokroot(x,k));
    }
}

