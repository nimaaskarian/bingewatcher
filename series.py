import sys
import os
import math

args=sys.argv[1:]

dir=os.path.expanduser("~/.series")

def getFile(name):
    return os.path.join(dir,name)
def convertToData(parsedData):
    return "\n".join([ f"{ l[0] }%:%+{ l[1] }" for l in parsedData])

def parseData(data):
    # [
    #         [watched, all],
    #         [watched, all]
    # ]
    return [[int(s) for s in line.split("%:%") ] for line in data.splitlines()]

def printParsed(parsedData):
    watched=0
    all=0
    for i,line in enumerate(parsedData):
        watched+=int(line[0])
        all+=int(line[1])
        print(f"Season {i+1}: {line[0]}/{line[1]}")
    print(f"Progress: {round(watched*1000/all)/10}%")
        

if not os.path.exists(dir):
    os.mkdir(dir)
definedArgs=["-c","-s","-n", "-d","-h"]
if not len(args) or "-h" in args:
    print('''Usage: series [OPTION...] [OPTION INPUTS]

Help Options:
    -h                                                          Shows help options 

Application Options:
    -n <series name> <seasons> <episodes>                       Initial a series with seasons and episodes.
    -s <series name>                                            Show a series progress
    -d <series name>                                            Delete a series
    -c <series name> <season> <episodes>                        Change a season to desired episodes
    <series name> <episodes count>                              Add or remove from watched.''')
    exit()
noargs = not len([x for x in args if x in definedArgs])

if noargs:
    file=getFile(args[0])
    times=1
    try:
        times=int(args[1])
    except IndexError as e:
        pass
    isNegative=abs(times) != times

    f=open(file,"r")
    parsedData = parseData(f.read())
    if isNegative: parsedData.reverse()
    f.close()
    for j in range(abs(times)):
        for i,l in enumerate(parsedData):
            if isNegative:
                if parsedData[i][0] == 0:
                    continue
                parsedData[i][0]-=1
            else:
                if l[0] != l[1]:
                    parsedData[i][0]+=1
                else: continue
            break
    if isNegative: parsedData.reverse()
    f=open(file,"w")
    data=convertToData(parsedData)
    try:
        f.write(data)
        printParsed(parseData(data))
    except Exception as e:
        raise e

if "-s" in args:
    index=args.index("-s")
    if index==len(args)-1:
        index=-1
    try:
        f=open(getFile(args[index+1]),"r")
        printParsed(parseData(f.read()))
            
    except Exception as e:
        raise e
if "-d" in args:
    index=args.index("-d")
    if index==len(args)-1:
        index=-1
    try:
        os.remove(getFile(args[index+1]))
    except Exception as e:
        raise e
if "-n" in args:
    index=args.index("-n")
    if index==len(args)-1:
        index=-1
    try:
        file=getFile(args[index+1])
        f = open(file, "x")
        f.close()
        for i in range(0, int(args[index+2])):
            f= open(file, "a")
            f.write(f"0%:%{args[index+3]}\n")
            f.close()
    except IndexError as e:
        pass
if "-c" in args:
    # -c serie 4 5
    # season 4, 5 episodes
    index=args.index("-c")
    if index==len(args)-1:
        index=-1
    try:
        file=getFile(args[index+1])
        f = open(file, "r")
        parsedData=parseData(f.read())
        f.close()
        indexArgument=int(args[index+2])-1
        episodes=int(args[index+3])

        if parsedData[indexArgument][0] > episodes:
            parsedData[indexArgument][0] = episodes

        parsedData[indexArgument][1] = episodes

        f= open(file, "w")
        data=convertToData(parsedData)
        try:
            f.write(data)
            printParsed(parsedData)
        except Exception as e:
            raise e
        f.close()
    except IndexError as e:
        print(e)
        pass
