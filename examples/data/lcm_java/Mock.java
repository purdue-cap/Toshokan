public class Library {
   public static int lcm(int a, int b) {
       if (a < b) {
           int swap = a;
           a = b;
           b = swap;
       }
       int gcd = 0;
       int product = a*b;
       while (b != 0) {
            gcd = b;
            b = a % b;
            a = gcd;
       }
       return product/gcd;
   }
}

@JavaCodeGen
public class LCM {
    static int start_i() {
        int i = ??;
        return i;
    }

    static int bound(int length) {
        int choice = ??;
        if (choice == 0)
            return length;
        else if (choice == 1)
            return length - 1;
        else if (choice == 2)
            return length - 2;
        assert false;
        return 0;
    }

    static int choice_a(int result, int other) {
        int choice = ??;
        if (choice == 0)
            return result;
        else if (choice == 1)
            return other;
        assert false;
        return 0;
    }

    static int choice_b(int result, int other) {
        int choice = ??;
        if (choice == 0)
            return result;
        else if (choice == 1)
            return other;
        assert false;
        return 0;
    }

    public static int lcm_n(int [] num) {
        int result = Library.lcm(num[0], num[1]);
        for (int i= start_i(); i < bound(num.length); i++) {
            int a = choice_a(result, num[i]);
            int b = choice_b(result, num[i]);
            result = Library.lcm(a, b);
        }
        return result;
    }
}

public class Main {
    harness public static void main(int x0, int x1, int x2, int x3, int x4, int k, int l) {
        assume x0 >=0 && x0 < 5;
        assume x1 >=0 && x1 < 5;
        assume x2 >=0 && x2 < 5;
        assume x3 >=0 && x3 < 5;
        assume x4 >=0 && x4 < 5;
        int [] array = new int[] {x0, x1, x2, x3, x4};
        for (int i = 0; i < array.length; i++)
            if (array[i]== 0) return;
        if (array.length < 2) return;
        int result = LCM.lcm_n(array);

        assume (k >=0 && k < array.length);
        assert result % array[k] == 0;

        assume (l >=1 && l < result);
        boolean divisible = true;
        for (int j = 0; j < array.length; j ++)
            divisible = divisible && (l % array[j] == 0);
        assert !divisible;
    }
}
