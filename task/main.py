import random

candies = [0]
types_amount = 24
remaining = [(i, j) for i in range(1, types_amount) for j in range(i)]

def add_candy(can, len_can, rem, len_rem, min_len_can):    
    random.shuffle(rem)
    if len_rem:
        if len_can + 1 == min_len_can:
            return min_len_can
        first = can[0]
        last = can[len_can - 1]
        added = False

        for i in range(len_rem):
            pair = rem[i]
            
            if pair[0] == last:
                cancopy = can[:]
                cancopy.append(pair[1])
                remcopy = rem[:i] + rem[i+1:]
                x = add_candy(cancopy, len_can + 1, remcopy, len_rem - 1, min_len_can)
                if min_len_can == -1 or x < min_len_can:
                    min_len_can = x
                added = True
            
            elif pair[1] == last:
                cancopy = can[:]
                cancopy.append(pair[0])
                remcopy = rem[:i] + rem[i+1:]
                x = add_candy(cancopy, len_can + 1, remcopy, len_rem - 1, min_len_can)
                if min_len_can == -1 or x < min_len_can:
                    min_len_can = x
                added = True
            
            elif pair[0] == first:
                cancopy = [pair[1]]
                cancopy.extend(can)
                remcopy = rem[:i] + rem[i+1:]
                x = add_candy(cancopy, len_can + 1, remcopy, len_rem - 1, min_len_can)
                if min_len_can == -1 or x < min_len_can:
                    min_len_can = x
                added = True
            
            elif pair[1] == first:
                cancopy = [pair[0]]
                cancopy.extend(can)
                remcopy = rem[:i] + rem[i+1:]
                x = add_candy(cancopy, len_can + 1, remcopy, len_rem - 1, min_len_can)
                if min_len_can == -1 or x < min_len_can:
                    min_len_can = x
                added = True
        
        if not added:
            for i in range(len_rem):
                pair = rem[i]
                for j in range(2):
                    cancopy = can[:]
                    cancopy.append(pair[j])
                    remcopy = rem[:]
                    x = add_candy(cancopy, len_can + 1, remcopy, len_rem, min_len_can)
                    if min_len_can == -1 or x < min_len_can:
                        min_len_can = x        
        
        return min_len_can    
    
    else:
        print(len_can, can)
        return len_can


print(add_candy(candies, 1, remaining, len(remaining), 620448401733239439360000))
