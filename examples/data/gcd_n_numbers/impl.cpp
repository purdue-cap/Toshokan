
int ANONYMOUS__gcd_real_impl(int a, int b){
	if(a < b) return ANONYMOUS__gcd_real_impl(b,a);
	int _gcd=0;
	while (b != 0) {
		_gcd = b;
		b = a % b;
		a = _gcd;
	}
	return _gcd;
}