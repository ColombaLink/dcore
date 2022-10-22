git commit -a -m "Release $1"
git push origin baselhack
git tag $1
git push origin $1
