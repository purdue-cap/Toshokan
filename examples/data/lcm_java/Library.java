public class Library {
   public static int lcm(int a, int b) {
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
       return b*a/gcd;
   }
}
