public class Library {
   public static int pow(int a, int b) {
        if (a > 1 && b < 0) {
            return 0;
        }
        int result = 1;
        for(int i=0; i<b; i++)
        result = result * a;
        return result;
   }
}

@JavaCodeGen
public class EvalPoly {
    static int expr_atom(int length, int choice, int hole) {
        if (choice == 0) {
            return length;
        } else if (choice == 1) {
            return hole;
        }
        assert false;
        return 0;
    }

    static int expr(int length) {
        int choice = ??;
        int choice_left = ??;
        int hole_left = ??;
        int choice_right = ??;
        int hole_right = ??;
        if (choice == 0) {
            return expr_atom(length, choice_left, hole_left);
        } else if (choice == 1) {
            return expr_atom(length, choice_left, hole_left) * expr_atom(length, choice_right, hole_right);
        } else if (choice == 2) {
            return expr_atom(length, choice_left, hole_left) + expr_atom(length, choice_right, hole_right);
        } else if (choice == 3) {
            return expr_atom(length, choice_left, hole_left) - expr_atom(length, choice_right, hole_right);
        }
        assert false;
        return 0;
    }

    public static int evalpoly(int[] p, int x) {
        int num = 0;
        int i = expr(p.length);
        while (i < p.length) {
            num += p[i] * Library.pow(x, i);
            i = i + 1;
        }
        return num;
    }
}

public class Main {
    harness public static void main(int p_0, int p_1, int p_2, int p_3, int x) {
        int[] p = new int[]{p_0, p_1, p_2, p_3};

        int num = EvalPoly.evalpoly(p, x);

        int ref_num = 0;
        for (int j=0; j < p.length; j++){
            ref_num += p[j] * Library.pow(x, j);
        }

        assert num == ref_num;

    }
}

