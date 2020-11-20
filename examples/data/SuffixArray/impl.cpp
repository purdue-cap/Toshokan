#include <stdlib.h>

namespace ANONYMOUS{
class TreeSet; 
class Array_char; 
class String; 
class Array_String; 
class TreeSet{
  public:
  Array_String*  set;
  int  capacity;
  int  size;
  TreeSet(){}
  static TreeSet* create(  Array_String*  set_,   int  capacity_,   int  size_);
  ~TreeSet(){
  }
  void operator delete(void* p){ free(p); }
};
class Array_char{
  public:
  int  length;
  char  A[];
  Array_char(){}
template<typename T_0>
  static Array_char* create(  int  length_,   T_0* A_, int A_len);
  ~Array_char(){
  }
  void operator delete(void* p){ free(p); }
};
class String{
  public:
  Array_char*  _value;
  int  _count;
  String(){}
  static String* create(  Array_char*  _value_,   int  _count_);
  ~String(){
  }
  void operator delete(void* p){ free(p); }
};
class Array_String{
  public:
  int  length;
  String*  A[];
  Array_String(){}
template<typename T_0>
  static Array_String* create(  int  length_,   T_0* A_, int A_len);
  ~Array_String(){
  }
  void operator delete(void* p){ free(p); }
};
}

using ANONYMOUS::TreeSet;
using ANONYMOUS::String;
using ANONYMOUS::Array_String;

void String_equals(String* self, String* s, bool& _out) {
  _out = 1;
  int  sLen_s126=s->_count;
  int  tLen_s128=self->_count;
  if ((sLen_s126) != (tLen_s128)) {
    _out = 0;
  }
  for (int  i=0;((i) < (sLen_s126)) && ((_out) == (1));i = i + 1){
    if (((s->_value->A[i])) != ((self->_value->A[i]))) {
      _out = 0;
    }
  }
  return;
}

void TreeSet_get_Index(TreeSet* self, String* o, int& _out) {
  bool  __sa3=(0) < (self->size);
  int  i=0;
  while (__sa3) {
    bool  _out_s124=0;
    String_equals(o, (self->set->A[i]), _out_s124);
    if (_out_s124) {
      _out = i;
      return;
    }
    i = i + 1;
    __sa3 = (i) < (self->size);
  }
  _out = -1;
  return;
}

void TreeSet_resize(TreeSet* self) {
  int  new_size=self->capacity * 2;
  Array_String*  new_set=Array_String::create(new_size, (String**)NULL, 0);
  bool  __sa2=(0) < (self->capacity);
  int  i=0;
  while (__sa2) {
    (new_set->A[i]) = (self->set->A[i]);
    i = i + 1;
    __sa2 = (i) < (self->capacity);
  }
  self->set = new_set;
  self->capacity = new_size;
}

void TreeSet_check_size(TreeSet* self) {
  if ((self->size) >= (self->capacity)) {
    TreeSet_resize(self);
  }
}

int ANONYMOUS__TreeSet_contains_real_impl(TreeSet* self, String* o) {
  int _out;
  int  _out_s122=0;
  TreeSet_get_Index(self, o, _out_s122);
  if ((_out_s122) >= (0)) {
    _out = 1;
    return _out;
  } else {
    _out = 0;
    return _out;
  }
}

int ANONYMOUS__TreeSet_add_real_impl(TreeSet* self, String* e) {
  int _out;
  int  _pac_sc_s139_s141=ANONYMOUS__TreeSet_contains_real_impl(self, e);
  bool  _pac_sc_s139=0;
  _pac_sc_s139 = (_pac_sc_s139_s141) == (1);
  if (!(_pac_sc_s139)) {
    _pac_sc_s139 = (e) == (NULL);
  }
  if (_pac_sc_s139) {
    _out = 0;
    return _out;
  } else {
    (self->set->A[self->size]) = e;
    self->size = self->size + 1;
    TreeSet_check_size(self);
    _out = 1;
    return _out;
  }
}

int ANONYMOUS__TreeSet_size_real_impl(TreeSet* self) {
  int _out = self->size;
  return _out;
}

void ANONYMOUS__TreeSet_clear_real_impl(TreeSet* self) {
  self->set = Array_String::create(16, (String**)NULL, 0);
  self->size = 0;
  self->capacity = 16;
}

int ANONYMOUS__String_HASHCODE(const String& str) {
if (str._count == 0) return 0;
if (str._value->A[0] == 'b') {
 if (str._count == 1) return 1;
 if (str._value->A[1] == 'a') {
  if (str._count == 2) return 2;
  if (str._value->A[2] == 'b') {
   if (str._count == 3) return 3;
   if (str._value->A[3] == 'a') {
    if (str._count == 4) return 4;
    if (str._value->A[4] == 'b') {
     if (str._count == 5) return 5;
     if (str._value->A[5] == 'a') {
      if (str._count == 6) return 6;
      if (str._value->A[6] == 'b') {
       if (str._count == 7) return 7;
       if (str._value->A[7] == 'a') {
        if (str._count == 8) return 8;
       }
      }
     }
    }
   }
  }
  if (str._value->A[2] == 'a') {
   if (str._count == 3) return 9;
   if (str._value->A[3] == 'b') {
    if (str._count == 4) return 10;
   }
  }
 }
 if (str._value->A[1] == 'c') {
  if (str._count == 2) return 11;
  if (str._value->A[2] == 'c') {
   if (str._count == 3) return 12;
   if (str._value->A[3] == 'd') {
    if (str._count == 4) return 13;
    if (str._value->A[4] == 'd') {
     if (str._count == 5) return 14;
    }
   }
  }
  if (str._value->A[2] == 'b') {
   if (str._count == 3) return 15;
   if (str._value->A[3] == 'a') {
    if (str._count == 4) return 16;
   }
  }
 }
}
if (str._value->A[0] == 'a') {
 if (str._count == 1) return 17;
 if (str._value->A[1] == 'b') {
  if (str._count == 2) return 18;
  if (str._value->A[2] == 'a') {
   if (str._count == 3) return 19;
   if (str._value->A[3] == 'b') {
    if (str._count == 4) return 20;
    if (str._value->A[4] == 'a') {
     if (str._count == 5) return 21;
     if (str._value->A[5] == 'b') {
      if (str._count == 6) return 22;
      if (str._value->A[6] == 'a') {
       if (str._count == 7) return 23;
      }
     }
    }
   }
   if (str._value->A[3] == 'a') {
    if (str._count == 4) return 24;
    if (str._value->A[4] == 'b') {
     if (str._count == 5) return 25;
    }
   }
  }
  if (str._value->A[2] == 'c') {
   if (str._count == 3) return 26;
   if (str._value->A[3] == 'c') {
    if (str._count == 4) return 27;
    if (str._value->A[4] == 'd') {
     if (str._count == 5) return 28;
     if (str._value->A[5] == 'd') {
      if (str._count == 6) return 29;
     }
    }
   }
   if (str._value->A[3] == 'b') {
    if (str._count == 4) return 30;
    if (str._value->A[4] == 'a') {
     if (str._count == 5) return 31;
    }
   }
  }
 }
 if (str._value->A[1] == 'a') {
  if (str._count == 2) return 32;
  if (str._value->A[2] == 'b') {
   if (str._count == 3) return 33;
   if (str._value->A[3] == 'a') {
    if (str._count == 4) return 34;
    if (str._value->A[4] == 'a') {
     if (str._count == 5) return 35;
     if (str._value->A[5] == 'b') {
      if (str._count == 6) return 36;
     }
    }
   }
  }
  if (str._value->A[2] == 'a') {
   if (str._count == 3) return 37;
   if (str._value->A[3] == 'b') {
    if (str._count == 4) return 38;
   }
  }
 }
}
if (str._value->A[0] == 'c') {
 if (str._count == 1) return 39;
 if (str._value->A[1] == 'c') {
  if (str._count == 2) return 40;
  if (str._value->A[2] == 'd') {
   if (str._count == 3) return 41;
   if (str._value->A[3] == 'd') {
    if (str._count == 4) return 42;
   }
  }
 }
 if (str._value->A[1] == 'd') {
  if (str._count == 2) return 43;
  if (str._value->A[2] == 'd') {
   if (str._count == 3) return 44;
  }
 }
 if (str._value->A[1] == 'b') {
  if (str._count == 2) return 45;
  if (str._value->A[2] == 'a') {
   if (str._count == 3) return 46;
  }
 }
}
if (str._value->A[0] == 'd') {
 if (str._count == 1) return 47;
 if (str._value->A[1] == 'd') {
  if (str._count == 2) return 48;
 }
}
return -1;

}
