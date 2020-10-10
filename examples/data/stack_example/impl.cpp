#include<cstdlib>
namespace ANONYMOUS{
class Stack; 
class Stack{
  public:
  int  pos;
  int  storage[];
  Stack(){}
template<typename T_0>
  static Stack* create(  T_0* storage_, int storage_len,   int  pos_);
  ~Stack(){
  }
  void operator delete(void* p){ free(p); }
};
};


int ANONYMOUS__s_push_real_impl(ANONYMOUS::Stack* s, int i) {
  int _out;
  if ((s->pos) >= (20)) {
    _out = 0;
    return _out;
  }
  (s->storage[s->pos]) = i;
  s->pos = s->pos + 1;
  _out = 1;
  return _out;
}

int ANONYMOUS__s_pop_real_impl(ANONYMOUS::Stack* s) {
  int _out;
  if ((s->pos) == (0)) {
    _out = 0;
    return _out;
  }
  s->pos = s->pos - 1;
  _out = (s->storage[s->pos]);
  return _out;
}