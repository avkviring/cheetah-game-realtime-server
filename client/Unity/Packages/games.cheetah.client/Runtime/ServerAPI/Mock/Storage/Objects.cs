using System;
using System.Collections.Generic;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock.Storage
{
    public class Objects : IObjectServerAPI
    {
        internal IObjectServerAPI.CreateListener createListener;
        internal IObjectServerAPI.CreatedListener createdListener;
        internal IObjectServerAPI.DeleteListener deleteListener;


        private readonly HashSet<CheetahObject> createObjects = new();
        internal readonly HashSet<CheetahCreatedObject> createdObjects = new();
        public ushort memberId = 0;
        private uint objectIdGenerator;


        internal struct CheetahCreatedObject : IEquatable<CheetahCreatedObject>
        {
            public CheetahObjectId objectId;
            public bool roomOwner;
            public CheetahBuffer singletonKey;

            public bool Equals(CheetahCreatedObject other)
            {
                return objectId.Equals(other.objectId) && roomOwner == other.roomOwner && singletonKey.Equals(other.singletonKey);
            }

            public override bool Equals(object obj)
            {
                return obj is CheetahCreatedObject other && Equals(other);
            }

            public override int GetHashCode()
            {
                unchecked
                {
                    var hashCode = objectId.GetHashCode();
                    hashCode = (hashCode * 397) ^ roomOwner.GetHashCode();
                    hashCode = (hashCode * 397) ^ singletonKey.GetHashCode();
                    return hashCode;
                }
            }
        }

        public byte SetCreateListener(ushort clientId, IObjectServerAPI.CreateListener listener)
        {
            createListener = listener;
            return 0;
        }

        public byte SetCreatedListener(ushort clientId, IObjectServerAPI.CreatedListener listener)
        {
            createdListener = listener;
            return 0;
        }

        public byte SetDeleteListener(ushort clientId, IObjectServerAPI.DeleteListener objectDeleteListener)
        {
            deleteListener = objectDeleteListener;
            return 0;
        }

        public byte CreateObject(ushort clientId, ushort template, ulong accessGroup, ref CheetahObjectId objectId)
        {
            objectId.memberId = memberId;
            objectId.id = objectIdGenerator++;
            createObjects.Add(new CheetahObject(objectId, template));
            return 0;
        }

        public byte CreatedObject(ushort clientId, in CheetahObjectId objectId, bool roomOwner, ref CheetahBuffer singletonKey)
        {
            var id = objectId;
            createObjects.RemoveWhere(item => item.ObjectId == id);
            createdObjects.Add(new CheetahCreatedObject
            {
                objectId = objectId,
                roomOwner = roomOwner,
                singletonKey = singletonKey
            });
            return 0;
        }

        public byte Delete(ushort clientId, in CheetahObjectId objectId)
        {
            var objectIdCloned = objectId;
            createObjects.RemoveWhere(item => item.ObjectId.Equals(objectIdCloned));
            createdObjects.RemoveWhere(item => item.objectId.Equals(objectIdCloned));
            return 0;
        }

        public void Clear()
        {
            createObjects.Clear();
            createdObjects.Clear();
        }


        public long GetCreatedObjectsCount()
        {
            return createdObjects.Count;
        }
    }
}