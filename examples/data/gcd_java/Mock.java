public class Library {
   public static int gcd(int a, int b) {
       if (a < b) {
           int swap = a;
           a = b;
           b = swap;
       }
       int gcd = 0;
       while (b != 0) {
            gcd = b;
            b = a % b;
            a = gcd;
       }
       return gcd;
   }
}

@JavaCodeGen
public class GCD {
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

    public static int gcd_n(int [] num) {
        int result = Library.gcd(num[0], num[1]);
        for (int i= start_i(); i < bound(num.length); i++) {
            int a = choice_a(result, num[i]);
            int b = choice_b(result, num[i]);
            result = Library.gcd(a, b);
        }
        return result;
    }
}

public class Main {
    harness public static void main(int x0, int x1, int x2, int x3, int x4) {
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
