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