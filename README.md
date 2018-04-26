# Elasticsearch CLI [![Build Status](https://travis-ci.org/avalarin/elasticsearch-cli.svg?branch=master)](https://travis-ci.org/avalarin/elasticsearch-cli)
Command-line interface for ElasticSearch

## Installing

### MacOS

TBD

### Linux

TBD

### From archive

* Download the latest release from the [releases page](https://github.com/avalarin/elasticsearch-cli/releases/latest)
* Unpack it to the executable files directory (e.g. /usr/local/bin)
* Make the kubectl binary executable `chmod +x /usr/local/bin/elastic-cli`

## Configuration

Configuration file stored in your home directory - `~/.elastic-cli`. Empty configuration file will be created at the first lanuch.

You need to register some elasticsearch server and set is as default:
```
eclastic-cli config add server local --address http://localhost:9200 --index '*'
eclastic-cli config use server local
```

## Usage

Examples:
```
eclastic-cli search -q 'level: Error'
eclastic-cli search -q 'level: Error' -o json
eclastic-cli search -q 'level: Error' -o '{level} {message}'
eclastic-cli search -q 'level: Error' -f 'level,message'
```

For more documentation use help:
```
eclastic-cli help
eclastic-cli search --help
eclastic-cli config --help
```
