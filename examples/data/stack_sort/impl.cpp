#include<cstdlib>
namespace ANONYMOUS{
class Stack; 
class Stack{
  public:
  int  pos;
  int  _hist_len;
  int*  storage;
  int  _hist[];
  Stack(){}
template<typename T_0, typename T_1>
  static Stack* create(  T_0* storage_, int storage_len,   int  pos_,   T_1* _hist_, int _hist_len,   int  _hist_len_);
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

int ANONYMOUS__s_peek_real_impl(ANONYMOUS::Stack* s) {
  int _out;
  if ((s->pos) == (0)) {
    _out = 0;
    return _out;
  }
  int idx = s->pos - 1;
  _out = (s->storage[idx]);
  return _out;
}

int ANONYMOUS__s_empty_real_impl(ANONYMOUS::Stack* s) {
  int _out;
  if ((s->pos) == (0)) {
    _out = 1;
    return _out;
  } else {
    return 0;
  }
}