# Photos Backend

This repository, which is still work in progress, will be a system
to collect data for training machine learning models on my private
photo collection.

## Browse

The first milestone would be to allow the user to efficiently browse
the pictures in the collection. These are high resolution pictures on
a Network Attached Storage (NAS), so browsing them through a common
file system protocol like SMB or NFS is too slow.

For the purpose of annotating them with ML data (categorize,
tag persons, etc.) one can reduce the quality of the pictures
(size and resolution) to make them more lightweight.
The backend will to this on the fly.

Together with this repository, a web frontend will be build in a separate
repository, that allows to browse through the pictures comfortably.

## Tag

Once I am able to browse the pictures efficiently, I will start annotating
them manually.

For now I'm thinking of two use cases: *classification* and *face recognition*.

### Classification

The goal is to be able to search for something like *all pictures of
landscapes in Abruzzo*. For this I need to be able to add tags to a picture.
Some tags can be derived from the folder structure (e.g. if a folder contains
photos of my vacation in Abruzzo, then all the pictures in the folder
will get the tag Abruzzo). The backend need to able to tag all pictures
in a folder with one API call.

### Face recognition

The goal is to be able to search for given persons (e.g. all pictures of me).
I will use a pre-trained model for face recognition, then do some clustering
and manually tag the clusters.

## Learn

## Improve
