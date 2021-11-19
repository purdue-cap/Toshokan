public class Main {
    static boolean primality_spec(int p){
        if(p<=1) return false;
        if(p==2) return true;
        for(int i=2;i<p;i++){
            if(p%i == 0) return false;
        }
        return true;
    }

    public static void main(int x) {
        assert Primality.primality(x) == primality_spec(x);
    }
}

