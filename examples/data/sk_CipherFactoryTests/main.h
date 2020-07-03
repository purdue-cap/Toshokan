#ifndef MAIN_H
#define MAIN_H

#include <cstdlib>

namespace StringBuilder{
}
namespace ICipherFactory{
}
namespace HashMap{
}
namespace ArrayList{
}
namespace CryptoManager{
}
namespace String{
}
namespace System{
}
namespace Assert{
}
namespace array{
class Array_bit; 
class Array_char; 
class Array_int; 
class Array_float; 
class Array_double; 
class Array_Object; 
}
namespace SecretKeySpec{
}
namespace List{
}
namespace Boolean{
}
namespace ANONYMOUS{
}
namespace Character{
}
namespace ICryptoManager{
}
namespace CharSequence{
}
namespace SecureRandom{
}
namespace HashMap_Node{
}
namespace Mac{
}
namespace Integer{
}
namespace Map_Entry{
}
namespace SecretKey{
}
namespace ConfigurableCipherFactory{
}
namespace meta{
}
namespace Byte{
}
namespace Object{
class Object; 
}
namespace IvParameterSpec{
}
namespace Cipher{
}
namespace CipherFactoryTests{
}
namespace DefaultCipherFactory{
}
namespace Map{
}
namespace Key{
}
namespace StringBuilder{
}
namespace ICipherFactory{
}
namespace HashMap{
extern void glblInit_DEFAULT_INITIAL_CAPACITY__HashMap_s1325(int& DEFAULT_INITIAL_CAPACITY__HashMap_s1324);
}
namespace ArrayList{
extern void glblInit_EMPTY_ELEMENTDATA__ArrayList_s1327(array::Array_Object*& EMPTY_ELEMENTDATA__ArrayList_s1326);
extern void glblInit_MAX_ARRAY_SIZE__ArrayList_s1329(int& MAX_ARRAY_SIZE__ArrayList_s1328);
}
namespace CryptoManager{
extern void CryptoManager_CryptoManager(Object::Object* self, Object::Object*& _out);
extern void encrypt_String(Object::Object* self, Object::Object* message, Object::Object*& _out);
extern void decrypt_String(Object::Object* self, Object::Object* encryptedMessage, Object::Object*& _out);
extern void readEncoded_String(Object::Object* self, Object::Object* encrypted, array::Array_char*& _out);
extern void getCipherFactory(Object::Object* self, Object::Object*& _out);
extern void appendEncryptionMark_byte(Object::Object* self, array::Array_char* bytesArray, array::Array_char*& _out);
extern void cryptInCipher_Cipher_byte(Object::Object* self, Object::Object* cipher, array::Array_char* data, array::Array_char*& _out);
extern void getCharset(Object::Object* self, Object::Object*& _out);
extern void decode_byte_String(Object::Object* self, array::Array_char* string, Object::Object* charset, Object::Object*& _out);
extern void isEncrypted_String(Object::Object* self, Object::Object* message, bool& _out);
extern void encode_String_String(Object::Object* self, Object::Object* string, Object::Object* charset, array::Array_char*& _out);
extern void isEncryptedByte_byte(Object::Object* self, array::Array_char* data, bool& _out);
extern void cutEncryptionMark_byte(Object::Object* self, array::Array_char* bytesArray, array::Array_char*& _out);
extern void getBasicCharset(Object::Object* self, Object::Object*& _out);
extern void processEscape_byte_boolean(Object::Object* self, array::Array_char* data, bool escape, array::Array_char*& _out);
extern void getEncryptedMark(Object::Object* self, char& _out);
extern void isUseEncryptionStrict(Object::Object* self, bool& _out);
}
namespace String{
extern void equals_Object(Object::Object* self, Object::Object* obj, bool& _out);
extern void String_String_char_int_int(Object::Object* self, array::Array_char* ca, int offset, int count, Object::Object*& _out);
extern void length(Object::Object* self, int& _out);
extern void String_String_byte(Object::Object* self, array::Array_char* bytes, Object::Object*& _out);
extern void getBytes(Object::Object* self, array::Array_char*& _out);
extern void toString(Object::Object* self, Object::Object*& _out);
extern void getBytes_String(Object::Object* str, array::Array_char*& _out);
extern void charAt_int(Object::Object* self, int index, char& _out);
}
namespace System{
extern void arraycopy_byte_int_byte_int_int(array::Array_char* src, int srcPos, array::Array_char* dst, int dstPos, int length);
}
namespace Assert{
}
namespace array{
class Array_bit; 
class Array_char; 
class Array_int; 
class Array_float; 
class Array_double; 
class Array_Object; 
class Array_bit{
  public:
  int  length;
  bool  A[];
  Array_bit(){}
template<typename T_0>
  static Array_bit* create(  int  length_,   T_0* A_, int A_len);
  ~Array_bit(){
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
class Array_int{
  public:
  int  length;
  int  A[];
  Array_int(){}
template<typename T_0>
  static Array_int* create(  int  length_,   T_0* A_, int A_len);
  ~Array_int(){
  }
  void operator delete(void* p){ free(p); }
};
class Array_float{
  public:
  int  length;
  float  A[];
  Array_float(){}
template<typename T_0>
  static Array_float* create(  int  length_,   T_0* A_, int A_len);
  ~Array_float(){
  }
  void operator delete(void* p){ free(p); }
};
class Array_double{
  public:
  int  length;
  double  A[];
  Array_double(){}
template<typename T_0>
  static Array_double* create(  int  length_,   T_0* A_, int A_len);
  ~Array_double(){
  }
  void operator delete(void* p){ free(p); }
};
class Array_Object{
  public:
  int  length;
  Object::Object*  A[];
  Array_Object(){}
template<typename T_0>
  static Array_Object* create(  int  length_,   T_0* A_, int A_len);
  ~Array_Object(){
  }
  void operator delete(void* p){ free(p); }
};
}
namespace SecretKeySpec{
extern void getEncoded(Object::Object* self, array::Array_char*& _out);
extern void SecretKeySpec_SecretKeySpec_byte_String(Object::Object* self, array::Array_char* key, Object::Object* type, Object::Object*& _out);
}
namespace List{
}
namespace Boolean{
}
namespace ANONYMOUS{
}
namespace Character{
}
namespace ICryptoManager{
}
namespace CharSequence{
}
namespace SecureRandom{
}
namespace HashMap_Node{
}
namespace Mac{
}
namespace Integer{
extern void toString_int(int i, Object::Object*& _out);
}
namespace Map_Entry{
}
namespace SecretKey{
}
namespace ConfigurableCipherFactory{
}
namespace meta{
extern void Object(int& _out);
extern void CryptoManager(int& _out);
extern void String(int& _out);
extern void DefaultCipherFactory(int& _out);
extern void ConfigurableCipherFactory(int& _out);
extern void Cipher(int& _out);
extern void SecretKeySpec(int& _out);
}
namespace Byte{
}
namespace Object{
class Object; 
class Object{
  public:
  int  __cid;
  bool  bool_Boolean;
  array::Array_Object*  elementData_ArrayList;
  int  DEFAULT_CAPACITY_ArrayList;
  int  capacity_ArrayList;
  int  size_ArrayList;
  Object*  key_HashMap_Node;
  Object*  value_HashMap_Node;
  int  hash_HashMap_Node;
  char  value_Character;
  Object*  type_Cipher;
  Object*  key_Cipher;
  int  mode_Cipher;
  array::Array_bit*  updated_Cipher;
  int  ENCRYPT_MODE_Cipher;
  int  DECRYPT_MODE_Cipher;
  array::Array_char*  key_SecretKeySpec;
  int  value_Integer;
  array::Array_char*  _value_String;
  int  _count_String;
  array::Array_char*  _value_StringBuilder;
  int  _count_StringBuilder;
  Object*  ALGORITHM_DefaultCipherFactory;
  Object*  PADDING_DefaultCipherFactory;
  Object*  algorithm_DefaultCipherFactory;
  Object*  padding_DefaultCipherFactory;
  Object*  key_DefaultCipherFactory;
  bool  keyBase64_DefaultCipherFactory;
  Object*  basicCharset_CryptoManager;
  Object*  charset_CryptoManager;
  char  encryptedMark_CryptoManager;
  bool  useEncryptionStrict_CryptoManager;
  Object*  cipherFactory_CryptoManager;
  array::Array_Object*  elementData_HashMap;
  int  numPairs_HashMap;
  int  capacity_HashMap;
  char  b_Byte;
  array::Array_bit*  _array_bit;
  array::Array_char*  _array_char;
  array::Array_int*  _array_int;
  array::Array_float*  _array_float;
  array::Array_double*  _array_double;
  array::Array_Object*  _array_object;
  bool  _bit;
  char  _char;
  int  _int;
  float  _float;
  double  _double;
  Object(){}
  static Object* create(  int  __cid_,   bool  bool_Boolean_,   array::Array_Object*  elementData_ArrayList_,   int  DEFAULT_CAPACITY_ArrayList_,   int  capacity_ArrayList_,   int  size_ArrayList_,   Object*  key_HashMap_Node_,   Object*  value_HashMap_Node_,   int  hash_HashMap_Node_,   char  value_Character_,   Object*  type_Cipher_,   Object*  key_Cipher_,   int  mode_Cipher_,   array::Array_bit*  updated_Cipher_,   int  ENCRYPT_MODE_Cipher_,   int  DECRYPT_MODE_Cipher_,   array::Array_char*  key_SecretKeySpec_,   int  value_Integer_,   array::Array_char*  _value_String_,   int  _count_String_,   array::Array_char*  _value_StringBuilder_,   int  _count_StringBuilder_,   Object*  ALGORITHM_DefaultCipherFactory_,   Object*  PADDING_DefaultCipherFactory_,   Object*  algorithm_DefaultCipherFactory_,   Object*  padding_DefaultCipherFactory_,   Object*  key_DefaultCipherFactory_,   bool  keyBase64_DefaultCipherFactory_,   Object*  basicCharset_CryptoManager_,   Object*  charset_CryptoManager_,   char  encryptedMark_CryptoManager_,   bool  useEncryptionStrict_CryptoManager_,   Object*  cipherFactory_CryptoManager_,   array::Array_Object*  elementData_HashMap_,   int  numPairs_HashMap_,   int  capacity_HashMap_,   char  b_Byte_,   array::Array_bit*  _array_bit_,   array::Array_char*  _array_char_,   array::Array_int*  _array_int_,   array::Array_float*  _array_float_,   array::Array_double*  _array_double_,   array::Array_Object*  _array_object_,   bool  _bit_,   char  _char_,   int  _int_,   float  _float_,   double  _double_);
  ~Object(){
  }
  void operator delete(void* p){ free(p); }
};
extern void Object_Object(Object* self, Object*& _out);
}
namespace IvParameterSpec{
}
namespace Cipher{
extern void Cipher_Cipher_String(Object::Object* self, Object::Object* type, Object::Object*& _out);
extern void doFinal_byte(Object::Object* self, array::Array_char* text, array::Array_char*& _out);
extern void getInstance_String(Object::Object* type, Object::Object*& _out);
extern void init_int_Key(Object::Object* self, int opmode, Object::Object* key);
}
namespace CipherFactoryTests{
extern void main__Wrapper(int p_i);
extern void main__WrapperNospec(int p_i);
extern void _main(int p_i);
}
namespace DefaultCipherFactory{
extern void DefaultCipherFactory_DefaultCipherFactory(Object::Object* self, Object::Object*& _out);
extern void encryptionCipher(Object::Object* self, Object::Object*& _out);
extern void decryptionCipher(Object::Object* self, Object::Object*& _out);
extern void initCipher_int(Object::Object* self, int mode, Object::Object*& _out);
extern void obtainCipher_int(Object::Object* self, int mode, Object::Object*& _out);
extern void key(Object::Object* self, array::Array_char*& _out);
extern void getAlgorithm(Object::Object* self, Object::Object*& _out);
extern void getPadding(Object::Object* self, Object::Object*& _out);
extern void getKey(Object::Object* self, Object::Object*& _out);
}
namespace Map{
}
namespace Key{
}

#endif
