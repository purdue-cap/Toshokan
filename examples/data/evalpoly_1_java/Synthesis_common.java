@JavaCodeGen
public class EvalPoly {
    static boolean cond_atom(int i, int choice, int bound) {
        if (choice == 0) {
            return i >= bound;
        } else if (choice == 1) {
            return i <= bound;
        } else if (choice == 2) {
            return i > bound;
        } else if (choice == 3) {
            return i < bound;
        }
        assert false;
        return false;
    }

    static boolean cond(int i) {
        int compound = ??;
        int choice_left = ??;
        int bound_left = ??;
        int choice_right = ??;
        int bound_right = ??;
        if (compound == 0) {
            return cond_atom(i, choice_left, bound_left);
        } else if (compound == 1) {
            return cond_atom(i, choice_left, bound_left) && cond_atom(i, choice_right, bound_right);
        } else if (compound == 2) {
            return cond_atom(i, choice_left, bound_left) || cond_atom(i, choice_right, bound_right);
        }
        assert false;
        return false;

    }
    public static int evalpoly(int[] p, int x) {
        int num = p[0];
        int i = p.length - 1;
        while (cond(i)) {
            num += p[i] * Library.pow(x, i);
            i = i - 1;
        }
        return num;
    }
}