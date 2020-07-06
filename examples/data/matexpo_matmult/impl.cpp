#include <cstdlib>
namespace ANONYMOUS{
class mat; 
class mat{
  public:
  int  R_0_0;
  int  R_0_1;
  int  R_1_0;
  int  R_1_1;
  int  k;
  mat(){}
  static mat* create(  int  R_0_0_,   int  R_0_1_,   int  R_1_0_,   int  R_1_1_,   int  k_);
  ~mat(){
  }
  void operator delete(void* p){ free(p); }
};
}

constexpr int n = 2;
constexpr int BASE = 5;
ANONYMOUS::mat ANONYMOUS__matmul_real_impl(ANONYMOUS::mat* A, ANONYMOUS::mat* B) {
    int R_rv[n][n];
    int R_A [n][n]= { {A->R_0_0, A->R_0_1}, {A->R_1_0, A->R_1_1} };
    int R_B [n][n]= { {B->R_0_0, B->R_0_1}, {B->R_1_0, B->R_1_1} };
	for(int i=0;i<n;i++){
		for(int j=0;j<n;j++){
			R_rv[i][j]=0;
			for (int k=0;k<n;k++){
				R_rv[i][j] = (R_rv[i][j] + (R_A[i][k] * R_B[k][j])%BASE)%BASE;
			}
		}
	}
    
	ANONYMOUS::mat* rv = ANONYMOUS::mat::create(R_rv[0][0],R_rv[0][1],R_rv[1][0],R_rv[1][1],1);
	return *rv;
}