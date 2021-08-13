using System.Collections;
using System.Runtime.ExceptionServices;
using System.Threading.Tasks;

namespace Tests
{
    public static class Enumerators
    {
        public static IEnumerator Await<T>(Task<T> task)
        {
            while (!task.IsCompleted)
            {
                yield return null;
            }

            if (task.IsFaulted)
            {
                ExceptionDispatchInfo.Capture(task.Exception).Throw();
            }

            yield return null;
        }
    }
}