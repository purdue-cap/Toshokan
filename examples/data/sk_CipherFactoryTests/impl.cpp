#include "main.h"

namespace SecretKeySpec{
void getEncoded(Object::Object* self, array::Array_char*& _out) {
  _out = self->key_SecretKeySpec;
  return;
}
}

array::Array_char* Cipher__doFinal_byte_impl(Object::Object* self, array::Array_char* text) {
  array::Array_char* _out;
  int  k_s1231=0;
  meta::SecretKeySpec(k_s1231);
  array::Array_char*  _pac_sc_s1232=NULL;
  if ((self->key_Cipher->__cid) == (k_s1231)) {
    array::Array_char*  k_s1234=NULL;
    SecretKeySpec::getEncoded(self->key_Cipher, k_s1234);
    _pac_sc_s1232 = k_s1234;
  } else {
    _pac_sc_s1232 = NULL;
  }
  _out = array::Array_char::create(text->length, (char*)NULL, 0);
  if ((_pac_sc_s1232->length) == (0)) {
    return _out;
  }
  if ((self->mode_Cipher) == (self->ENCRYPT_MODE_Cipher)) {
    bool  __sa7=(0) < (text->length);
    int  i=0;
    while (__sa7) {
      (_out->A[i]) = (text->A[i]) + (_pac_sc_s1232->A[(i % _pac_sc_s1232->length)]);
      i = i + 1;
      __sa7 = (i) < (text->length);
    }
  } else {
    if ((self->mode_Cipher) == (self->DECRYPT_MODE_Cipher)) {
      bool  __sa8=(0) < (text->length);
      int  i_0=0;
      while (__sa8) {
        (_out->A[i_0]) = (text->A[i_0]) - (_pac_sc_s1232->A[(i_0 % _pac_sc_s1232->length)]);
        i_0 = i_0 + 1;
        __sa8 = (i_0) < (text->length);
      }
    }
  }
  return _out; 
}
