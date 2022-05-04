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
