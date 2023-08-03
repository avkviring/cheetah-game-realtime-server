using System;
using Unity.Collections;

namespace Games.Cheetah.Client.Tests.Server
{
    public static class NativeListExt
    {
        public static T SearchFirst<T>(this NativeList<T> list, Predicate<T> predicate) where T : unmanaged
        {
            foreach (var item in list)
            {
                if (predicate.Invoke(item))
                {
                    return item;
                }
            }

            throw new Exception("Not found");
        }

        public static T SearchLast<T>(this NativeList<T> list, Predicate<T> predicate) where T : unmanaged
        {
            for (var i = list.Length - 1; i >= 0; i--)
            {
                var item = list[i];
                if (predicate.Invoke(item))
                {
                    return item;
                }
            }

            throw new Exception("Not found");
        }
    }
}