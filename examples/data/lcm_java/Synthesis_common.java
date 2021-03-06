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