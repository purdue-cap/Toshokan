#!/usr/bin/env python3

import argparse
import sys

SK_CHECK = "String_charAt(str, {index}) == '{char}'"
SK_TERM = "String_length(str) == {index}"
CPP_CHECK = "str._value->A[{index}] == '{char}'"
CPP_TERM = "str._count == {index}"
SK_SIGNATURE = "int String_hashCode(String str)"
CPP_SIGNATURE = "int ANONYMOUS__String_HASHCODE(const String& str)"

class TrieNode(object):
    def __init__(self):
        self.children = {}
    
    def insert(self, string):
        if not string:
            return 
        char = string[0]
        remaining = string[1:]
        if not char in self.children:
            self.children[char] = TrieNode()
        self.children[char].insert(remaining)
    
    def print_tree(self, indent = 0):
        for char, child in self.children.items():
            print(" " * indent + "- " + char)
            child.print_tree(indent + 1)
        
    def generate_code(self, check_tplt, term_tplt, indent = "", depth = 0, code = 0):
        generated_code = indent*depth + "if ({term}) return {code};\n".format(term = term_tplt.format(index = depth), code = code)
        visited_nodes = 1
        for i, (char, child) in enumerate(self.children.items()):
            generated_code += indent*depth + "if ({}) {{\n".format(check_tplt.format(index = depth, char = char))
            inner_code, inner_visited = child.generate_code(check_tplt, term_tplt, indent, depth + 1, code + visited_nodes)
            generated_code += inner_code
            visited_nodes += inner_visited
            generated_code += indent*depth + "}\n"
        return (generated_code, visited_nodes)




parser = argparse.ArgumentParser()

parser.add_argument("strings", nargs="+", help="Whole strings for hash function to consider as samples")
parser.add_argument("-l", "--lang", choices=["sk", "cpp", "trie"], default="sk", help="Target language to generate")
parser.add_argument("--sk_check", default=SK_CHECK, help="Template to generate if checks in sk code")
parser.add_argument("--cpp_check", default=CPP_CHECK, help="Template to generate if checks in cpp code")
parser.add_argument("--sk_term", default=SK_TERM, help="Template to generate termination checks in sk code")
parser.add_argument("--cpp_term", default=CPP_TERM, help="Template to generate termination checks in cpp code")
parser.add_argument("--sk_signature", default=SK_SIGNATURE, help="Hash function signature used in sk code")
parser.add_argument("--cpp_signature", default=CPP_SIGNATURE, help="Hash function signature used in cpp code")
parser.add_argument("-i", "--indent", type=int, default=0, help="Number of space indents in generated code")
parser.add_argument("-d", "--default", type=int, default=-1, help="Default return value if no matching cases are found")

args = parser.parse_args()

string_samples = set(args.strings)
processed_substrings = set()

trie = TrieNode()

if args.lang == "sk":
    check_tplt = args.sk_check
    term_tplt = args.sk_term
    signature_tplt = args.sk_signature
elif args.lang == "cpp":
    check_tplt = args.cpp_check
    term_tplt = args.cpp_term
    signature_tplt = args.cpp_signature
elif args.lang == "trie":
    check_tplt = None
    term_tplt = None 
    signature_tplt = None
else:
    raise Exception()

for string_sample in string_samples:
    for i in range(len(string_sample)):
        for j in range(i, len(string_sample)):
            trie.insert(string_sample[i:j+1])

if args.lang == "trie":
    trie.print_tree()
    exit(0)

generated_code, visited = trie.generate_code(check_tplt, term_tplt, indent = " "*args.indent)
generated_code += "return {};\n".format(args.default)

generated_code = "{signature} {{\n{content}\n}}".format(signature=signature_tplt, content=generated_code)
print(generated_code)
print("Total processed substrings: {}".format(visited), file=sys.stderr)