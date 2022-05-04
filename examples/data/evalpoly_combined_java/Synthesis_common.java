@JavaCodeGen
public class EvalPoly {
    static boolean cond_atom(int i, int length, int choice, int bound) {
        if (choice == 0) {
            return i >= bound;
        } else if (choice == 1) {
            return i <= bound;
        } else if (choice == 2) {
            return i > bound;
        } else if (choice == 3) {
            return i < bound;
        } else if (choice == 4) {
            return i >= length;
        } else if (choice == 5) {
            return i <= length;
        } else if (choice == 6) {
            return i > length;
        } else if (choice == 7) {
            return i < length;
        }
        assert false;
        return false;
    }

    static boolean cond(int i, int length) {
        int compound = ??;
        int choice_left = ??;
        int bound_left = ??;
        int choice_right = ??;
        int bound_right = ??;
        if (compound == 0) {
            return cond_atom(i, length, choice_left, bound_left);
        } else if (compound == 1) {
            return cond_atom(i, length, choice_left, bound_left) && cond_atom(i, length, choice_right, bound_right);
        } else if (compound == 2) {
            return cond_atom(i, length, choice_left, bound_left) || cond_atom(i, length, choice_right, bound_right);
        }
        assert false;
        return false;

    }
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
        while (cond(i, p.length)) {
            num += p[i] * Library.pow(x, i);
            i = i - 1;
        }
        return num;
    }
}