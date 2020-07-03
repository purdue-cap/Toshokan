#include<vector>

constexpr int n = 5;
constexpr int BASE = 4;

std::vector<int> ANONYMOUS__mult_proxy_impl(
    int x_0,int x_1,int x_2,int x_3,int x_4,
    int y_0,int y_1,int y_2,int y_3,int y_4
) {
    std::vector<int> x = {x_0,x_1,x_2,x_3,x_4};
    std::vector<int> y = {y_0,y_1,y_2,y_3,y_4};
    std::vector<int> out = {0,0,0,0,0,0,0,0,0,0};
    for(int i=0; i<n; ++i){
        for(int j=0; j<n; ++j){            
            int tmp = y[i] * x[j];
            tmp = out[j + i] + tmp;
            out[j + i] = tmp % BASE;
            out[j + i + 1] = out[j + i + 1] + (tmp / BASE); 
        }           
    }       
    return out;	
}
std::vector<int> toBase(
    int z
) {
    std::vector<int> out = {0,0,0,0,0};
	int sum =z;
	for (int i=0;i<n;i++){
		out[i] = sum % BASE;
		sum = sum/BASE;
	}
	return out;
}

std::vector<int> ANONYMOUS__exp_impl(
    int x, int i
) {
	int res = 1;
	for(int j=0;j<i;j++){
		res = res * x;
	}
	return toBase(res);	
}
