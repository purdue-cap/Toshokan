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
public class Primality {
    static int get_arg(int p) {
        int factor = ??%2;
        int offset = ??%2;
        int symbol = ??%2;
        if (symbol == 0) {
            return factor * p + offset;
        } else if (symbol == 1) {
            return factor * p - offset;
        }
        assert false;
        return 0;
    }
    static boolean cond(int i, int temp) {
        int comparison = ??%4;
        if (comparison == 0) {
            return i < temp;
        } else if (comparison == 1) {
            return i > temp;
        } else if (comparison == 2) {
            return i <= temp;
        } else if (comparison == 3) {
            return i >= temp;
        }
        assert false;
        return false;

    }
    public static boolean primality(int p) {
        if (p<=1) return false;
        if (p==2) return true;
        int temp = Library.sqrt(get_arg(p));
        for (int i = 2; cond(i, temp); i++) {
            if (p%i == 0) return false;
        }
        return true;
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
        assert Primality.primality(x) == primality_spec(x);
    }
}

