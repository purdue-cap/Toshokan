
int gcd(int a, int b){
	if(a < b) return gcd(b,a);
	int _gcd=0;
	while (b != 0) {
		_gcd = b;
		b = a % b;
		a = _gcd;
	}
	return _gcd;
}

int ANONYMOUS__lcm_real_impl(int a, int b){
	return a * b / gcd(a,b);
}