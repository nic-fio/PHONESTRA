#!/bin/sh
# Compila l'aiutante di Phonestra in telefono/phonestra-aiuto.jar (classes.dex).
# Serve Java (javac) e D8 di R8 9.4.26 in strumenti/r8.jar:
#   curl -L -o strumenti/r8.jar https://dl.google.com/android/maven2/com/android/tools/r8/9.4.26/r8-9.4.26.jar
#   sha256: 870354f719912241712d6773bfd7c66b1138b790a1ee547dc4726c6ddcf2c332
# Le classi in stub/ imitano le API Android usate: servono solo a javac, non
# finiscono nel .jar (sul telefono ci sono quelle vere).
set -eu
qui=$(cd "$(dirname "$0")" && pwd)
radice=$(cd "$qui/../.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
javac --release 11 -nowarn -d "$tmp/stub" $(find "$qui/stub" -name '*.java')
javac --release 11 -cp "$tmp/stub" -d "$tmp/classi" $(find "$qui/src" -name '*.java')
# Anche le classi del JDK come libreria: D8 deve conoscere java.lang.Object
# per le nostre sottoclassi di classi Android (TaskStackListener).
jdk=$(dirname "$(dirname "$(readlink -f "$(command -v javac)")")")
java -cp "$radice/strumenti/r8.jar" com.android.tools.r8.D8 --release --min-api 34 \
    --lib "$jdk" --lib "$tmp/stub" --output "$radice/telefono/phonestra-aiuto.jar" $(find "$tmp/classi" -name '*.class')
echo "creato telefono/phonestra-aiuto.jar"
