using System.IO;
using UnityEngine;

namespace Cheetah.Platform.Editor.LocalServer
{
    public static class PlatformConfiguration
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

        public static void InitConfigDirectoryIfNotExists(string sourcePackage, string destinationPath)
        {
            var fullDestinationPath = MakeHostedDataPath(destinationPath);
            if (Directory.Exists(fullDestinationPath)) return;
            Directory.CreateDirectory(fullDestinationPath);
            var sourcePath = Path.GetFullPath("Packages/" + sourcePackage + "/ConfigTemplates/");
            CopyDirectory(sourcePath, fullDestinationPath);
        }

        private static void CopyDirectory(string sourceDir, string destinationDir)
        {
            Debug.Log("Copy " + sourceDir + " ===> " + destinationDir);
            var dir = new DirectoryInfo(sourceDir);
            if (!dir.Exists)
                throw new DirectoryNotFoundException($"Source directory not found: {dir.FullName}");
            var dirs = dir.GetDirectories();
            Directory.CreateDirectory(destinationDir);
            foreach (var file in dir.GetFiles())
            {
                var targetFilePath = Path.Combine(destinationDir, file.Name);
                file.CopyTo(targetFilePath);
            }

            foreach (var subDir in dirs)
            {
                var newDestinationDir = Path.Combine(destinationDir, subDir.Name);
                CopyDirectory(subDir.FullName, newDestinationDir);
            }
        }
    }
}