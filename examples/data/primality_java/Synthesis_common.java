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