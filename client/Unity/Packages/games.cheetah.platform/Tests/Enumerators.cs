using System.Collections;
using System.Threading.Tasks;
using UnityEngine;

namespace Cheetah.Platform.Tests
{
    public static class Enumerators
    {
        public static IEnumerator Await<T>(Task<T> task)
        {
            while (!task.IsCompleted) yield return null;

            if (task.IsFaulted)
            {
                Debug.LogError(task.Exception);
                yield break;
            }

            yield return null;
        }

        public static IEnumerator Await(Task task)
        {
            while (!task.IsCompleted) yield return null;

            if (task.IsFaulted)
            {
                Debug.LogError(task.Exception);
                yield break;
            }

            yield return null;
        }
    }
}