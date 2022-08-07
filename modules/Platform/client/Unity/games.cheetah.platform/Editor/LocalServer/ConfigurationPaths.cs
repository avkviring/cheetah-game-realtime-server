using UnityEngine;

namespace Cheetah.Platform.Editor.LocalServer
{
    public static class ConfigurationPaths
    {
        /// <summary>
        /// Путь до файлов проекта, которые используются только для локальной разработки
        /// </summary>
        /// <param name="project"></param>
        /// <returns></returns>
        public static string MakeLocalDataPath(string project)
        {
            return Application.dataPath + "/Editor/Cheetah/Local/" + project + "/";
        }

        /// <summary>
        /// Путь до файлов, необходимых для запуска сервиса в на внешнем хостинге
        /// </summary>
        /// <param name="project"></param>
        /// <returns></returns>
        public static string MakeHostedDataPath(string project)
        {
            return Application.dataPath + "/Editor/Cheetah/Production/" + project + "/";
        }
    }
}