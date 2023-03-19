import sys

def valid_word(word):
    if not word.isalnum():
        return False
    
    l = len(word)
    if not (4 <= l <= 15):
        return False

    return True

def main():
    
    filename = sys.argv[1]
    target_size = int(sys.argv[2])
    
    if not (4 <= target_size <= 15):
        print("target size is invalid")
        return 


    try:
        with open(filename, 'r') as f:
            
            bank = []
            bank_map = dict()
            for l in f:
                l_str = l.strip()
                l_str = l_str.lower()
                if valid_word(l_str):
                    bank.append(l_str)
                    l_len = len(l_str)
                    bucket = bank_map.get(l_len, [])
                    bucket.append(l_str)
                    bank_map[l_len] = bucket
            
            # print(len(bank))
            # for k,v in bank_map.items():
            #     print(f"there are {len(v)} word(s) of size {k}")
            
            print("\n".join(bank_map[target_size]))
    except:
        print("error occured")



    pass

if __name__ == "__main__":
    main()
