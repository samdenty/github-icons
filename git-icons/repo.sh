set -e

repo_dir=${1%%.git}
cd $repo_dir

repo=$(git config --get remote.origin.url)

# extract the protocol
proto="$(echo $repo | grep :// | sed -e's,^\(.*://\).*,\1,g')"
# remove the protocol
url="$(echo ${repo/$proto/})"
# extract the user (if any)
user="$(echo $url | grep @ | cut -d@ -f1)"
# extract the host and port
hostport="$(echo ${url/$user@/} | cut -d/ -f1)"
# by request host without port
host="$(echo $hostport | sed -e 's,:.*,,g')"
# by request - try to extract the port
port="$(echo $hostport | sed -e 's,^.*:,:,g' -e 's,.*:\([0-9]*\).*,\1,g' -e 's,[^0-9],,g')"
# extract the path (if any)
path="$(echo $url | grep / | cut -d/ -f2-)"
# extract the slug
slug="${path%%.git}"
slug="${slug%%/}"

if [[ "$host" != "github.com" ]]; then
  exit
fi

echo $slug

icon=/tmp/${slug//\//-}
rsrc=/tmp/${slug//\//-}.rsrc

if [ ! -f $icon.png ]; then
  icons=$(repo-icons $slug --token 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)
  # get the first icon
  icon_url=${icons%% *}

  if [[ $icon_url == data:* ]]; then
    kind=${icon_url#*:}
    kind=${kind%;*}

    type=${icon_url#*;}
    type=${type%,*}

    data=${icon_url#*,}

    if [[ $type == "base64" ]]; then
      echo $data | base64 -d >$icon
    fi
  else
    curl -s $icon_url --output $icon
  fi

  convert -density 1200 -resize 1024x1024 -thumbnail '1024x1024>' \
    -background "rgba(0,0,0,0.003)" -gravity center \
    -extent 1024x1024 $icon $icon.png

  rm $icon
fi

# add the icon flag to directory
SetFile -a C .

# Add icon to image file, meaning use itself as the icon
sips -i $icon.png >/dev/null

# Take that icon and put it into a rsrc file
DeRez -only icns $icon.png >$rsrc

# Create the magical Icon\r file
touch $'Icon\r'
Rez -append $rsrc -o Icon?
SetFile -a V Icon?

rm $rsrc
