git commit -a -m "Release $1"
git tag $1
git push origin baselhack
git push origin $1
