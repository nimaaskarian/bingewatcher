#!/bin/python3

import sys
import os
import requests
import json

args=sys.argv[1:]

dir=os.path.expanduser("~/.series")
jsondir=os.path.join(dir,".json")

def listdir():
    return [x for x in os.listdir(dir) if os.path.isfile(os.path.join(dir,x)) ]

def newSeries(seasons, episodes):
    return [[0,int(episodes)] for s in range(int(seasons)) ]

def getName(nameOrIndex):
    try:
        if int(nameOrIndex): nameOrIndex = listdir()[int(nameOrIndex)-1]
    except:
        pass
    return nameOrIndex

def getFile(nameOrIndex,dir=dir):
    return os.path.join(dir,getName(nameOrIndex))
def convertToData(parsedData):
    return "\n".join([ f"{ l[0] }%:%+{ l[1] }" for l in parsedData])

def nextEpisode(parsedData,formatWidth=2):
    lastSeason = len([x for x in parsedData if x[0] != 0])
    if lastSeason == 0: lastSeason = 1
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

def printParsed(parsedData,nameOrIndex=""):
    if nameOrIndex:
        print(getName(nameOrIndex)+":")
    for i,line in enumerate(parsedData):
        if line[0]!=line[1]:
            print(f"Season {i+1}: {line[0]}/{line[1]}")
            if not "-x" in args: break
    watchedAll = getWatchedAll(parsedData)
    print(f"Episodes: {watchedAll[1]}")
    print(f"Progress: {getPercentage(*watchedAll)}%")
    print(f"Next episode: {nextEpisode(parsedData)}")
        

if not os.path.exists(dir):
    os.mkdir(dir)
if not os.path.exists(jsondir):
    os.mkdir(jsondir)
definedArgs=["-l","-c","-s","-n", "-d","-h","-L","-x","-e","-o"]
if "-h" in args:
    print('''Usage: series [OPTION...] [OPTION INPUTS]

Help Options:
    -h                                                          Shows help options 

Application Options:
    -n <series name> <seasons> <episodes>                       Initialize a series with seasons and episodes
    -s <series name>                                            Show a series progress
    -d <series name>                                            Delete a series
    -c <series name> <season> <episodes>                        Change a season to desired episodes
    -l                                                          List all the series
    -L                                                          Show all the series
    -x                                                          Expanded show (show other seasons)
    -e                                                          Output the current episode without newlines
    -o                                                          Initialize or update a series with online api
    <series name> <episodes count>                              Add or remove from watched.''')
    exit()
noargs = not len([x for x in args if x in definedArgs and x!="-x"]) and len(args)
cleanargs=[arg for arg in args if arg not in definedArgs]

if noargs and len(cleanargs):
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
        printParsed(parseData(data),cleanargs[0])
    except Exception as e:
        raise e

if "-s" in args:
    index=args.index("-s")
    if index==len(args)-1:
        index=-1
    try:
        f=open(getFile(args[index+1]),"r")
        printParsed(parseData(f.read()),args[index+1])
            
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
        if not os.path.isfile(file):
            f = open(file, "x")
            f.close()
        f= open(file, "w")
        f.write(convertToData(newSeries(*args[2:])))
        f.close()
    except IndexError as e:
        pass
if "-o" in args:
    index=args.index("-o")
    if index==len(args)-1:
        index=-1
    # try:
    item=args[index+1]
    x = requests.get(f"https://www.episodate.com/api/show-details?q={item.replace(' ','-')}")
    file=getFile(item)
    parsedData = []
    if not os.path.isfile(file):
        f = open(file, "x")
        f.close()
    else:
        f = open(file, "r")
        parsedData = parseData(f.read())
        for i in range(len(parsedData)):
            parsedData[i][1] = 0
    for e in json.loads(x.text)["tvShow"]["episodes"]:
        index=int(e['season'])-1;
        try:
            parsedData[index]
        except IndexError:
            while (index != len(parsedData)-1):
                parsedData.append([0,0])
        parsedData[index][1] += 1
    f= open(file, "w")
    f.write(convertToData(parsedData))
    f.close()
    jsonfile=getFile(item,jsondir)
    if not os.path.isfile(jsonfile):
        f = open(jsonfile, "x")
        f.close()
    f= open(jsonfile, "w")
    f.write(x.text)
    f.close()
    printParsed(parsedData)
    # except IndexError as e:
        # pass
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
            printParsed(parsedData,args[index+1])
        except Exception as e:
            raise e
        f.close()
    except IndexError as e:
        print("Invalid arguments!")
if not len(args) or "-l" in args:
    for item in listdir():
        f = open(getFile(item),"r")
        parsedData = parseData((f.read()))
        watchedAll = getWatchedAll(parsedData)
        print(item+":")
        print("{all} episodes, {percentage}% watched. next is {next}".format(item=item, all=watchedAll[1],percentage=getPercentage(*watchedAll),next=nextEpisode(parsedData)))
        # print(f"{item}, {watchedAll[0]} episodes, {getPercentage(*watchedAll)}% watched. next is S{lastSeason}E{parsedData[lastSeason-1][0]}")
        print()
if "-L" in args:
    for item in listdir():
        f = open(getFile(item),"r")
        parsedData = parseData((f.read()))
        printParsed(parsedData,item)
        print()
if "-e" in args:
    index=args.index("-e")
    if index==len(args)-1:
        index=-1
    print(nextEpisode(parseData(open(getFile(args[index+1])).read())),end="")
