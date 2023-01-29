#!/bin/python3

import sys
import os

args=sys.argv[1:]

dir=os.path.expanduser("~/.series")

def getFile(name):
    return os.path.join(dir,name)
def convertToData(parsedData):
    return "\n".join([ f"{ l[0] }%:%+{ l[1] }" for l in parsedData])

def nextEpisode(parsedData,formatWidth=2):
    lastSeason = len([x for x in parsedData if x[0] != 0])
    return "S{seasons:0{width}d}E{episodes:0{width}d}".format(seasons=lastSeason,episodes=parsedData[lastSeason-1][0]+1,width=formatWidth)

def getWatchedAll(parsedData):
    watched=0
    all=0
    for line in parsedData:
        watched+=int(line[0])
        all+=int(line[1])
    return [watched,all]

def getPercentage(part,whole):
    return round(part*1000//whole)/10


def parseData(data):
    # [
    #         [watched, all],
    #         [watched, all]
    # ]
    return [[int(s) for s in line.split("%:%") ] for line in data.splitlines()]

def printParsed(parsedData):
    for i,line in enumerate(parsedData):
        if ( not "-m" in args ) or line[0]!=line[1]:
            print(f"Season {i+1}: {line[0]}/{line[1]}")
            if "-m" in args: break
    watchedAll = getWatchedAll(parsedData)
    print(f"Progress: {getPercentage(*watchedAll)}%")
    print(f"Next episode: {nextEpisode(parsedData)}")
        

if not os.path.exists(dir):
    os.mkdir(dir)
definedArgs=["-l","-c","-s","-n", "-d","-h","-L","-m","-e"]
if "-h" in args:
    print('''Usage: series [OPTION...] [OPTION INPUTS]

Help Options:
    -h                                                          Shows help options 

Application Options:
    -n <series name> <seasons> <episodes>                       Initial a series with seasons and episodes.
    -s <series name>                                            Show a series progress
    -d <series name>                                            Delete a series
    -c <series name> <season> <episodes>                        Change a season to desired episodes
    -l                                                          List all the series
    -L                                                          Show all the series
    -m                                                          Minimal show (only show current season)
    -e                                                          Output the current episode without newlines
    <series name> <episodes count>                              Add or remove from watched.''')
    exit()
noargs = not len([x for x in args if x in definedArgs and x!="-m"]) and len(args)
cleanargs=[arg for arg in args if arg not in definedArgs]

if noargs:
    file=getFile(cleanargs[0])
    times=0
    try:
        times=int(cleanargs[1])
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
        print("Invalid arguments!")
if not len(args) or "-l" in args:
    for item in os.listdir(dir):
        f = open(getFile(item),"r")
        parsedData = parseData((f.read()))
        watchedAll = getWatchedAll(parsedData)
        print(item+":")
        print("{all} episodes, {percentage}% watched. next is {next}".format(item=item, all=watchedAll[0],percentage=getPercentage(*watchedAll),next=nextEpisode(parsedData)))
        # print(f"{item}, {watchedAll[0]} episodes, {getPercentage(*watchedAll)}% watched. next is S{lastSeason}E{parsedData[lastSeason-1][0]}")
        print()
if "-L" in args:
    for item in os.listdir(dir):
        f = open(getFile(item),"r")
        parsedData = parseData((f.read()))
        print(item+":")
        printParsed(parsedData)
        print()
if "-e" in args:
    index=args.index("-e")
    if index==len(args)-1:
        index=-1
    print(nextEpisode(parseData(open(getFile(args[index+1])).read())),end="")
