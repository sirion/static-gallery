package main

import (
	"bytes"
	"encoding/json"
	"flag"
	"fmt"
	"image"
	"image/jpeg"
	"io"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"

	"github.com/nfnt/resize"
	"golang.org/x/net/html"
)

// Some often used resolutions
const (
	// 16:9
	ResolutionQuarterFullHD = "960x540"
	ResolutionQuadHD        = "2560x1440"
	ResolutionFullHD        = "1920x1080"
	ResolutionUHD           = "3840x2160"
)

var imageExtensions = map[string]bool{
	".jpg": true, ".jpeg": true,
}

type size struct {
	width  int
	height int
}

type collection struct {
	Title       string         `json:"title"`
	Pictures    []pictureEntry `json:"pictures"`
	Backgrounds []string       `json:"backgrounds"`

	files []string
}

type pictureEntry struct {
	Picture  string `json:"picture"`
	Fullsize string `json:"fullsize,omitempty"`
	Thumb    string `json:"thumb,omitempty"`
	Title    string `json:"title,omitempty"`
}

func main() {
	var outDir, templatePath, thumbSizeStr, displaySizeStr, backgroundSizeStr string
	var optimize bool

	flag.StringVar(&outDir, "output", "", "Directory where the generated gallery is written to")

	flag.StringVar(&templatePath, "template", "", "Directory containing the template")

	flag.BoolVar(&optimize, "optimize", false, "Embed JavaScript and CSS into the HTML file and remove comments")

	flag.StringVar(&thumbSizeStr, "thumbSize", ResolutionQuarterFullHD, "Maximum size of thumbnail pictures")
	flag.StringVar(&displaySizeStr, "displaySize", ResolutionQuadHD, "Maximum size of display pictures")
	flag.StringVar(&backgroundSizeStr, "backgroundSize", ResolutionQuadHD, "Maximum size of display pictures")
	help := flag.Bool("help", false, "Show this help")
	flag.Parse()

	if *help {
		showHelp()
	}

	// TODO: Maybe include templates in binary? :)
	if templatePath == "" {
		exitApplication(36, "Please provide a valid template path")
	}

	validateOutputDir(outDir)

	// Rest of the arguments
	dirs := flag.Args()

	thumbSize, err := parseSize(thumbSizeStr)
	if err != nil {
		exitApplication(1, err.Error())
	}
	displaySize, err := parseSize(displaySizeStr)
	if err != nil {
		exitApplication(2, err.Error())
	}
	backgroundSize, err := parseSize(backgroundSizeStr)
	if err != nil {
		exitApplication(3, err.Error())
	}

	if len(dirs) != 1 {
		exitApplication(4, "Please provide a directory consisting of background pictures and folders containing collection pictures")
	}
	dir := dirs[0]

	var backgrounds []string
	var collections []*collection

	// Validate collection folder
	folders := make([]string, 0)
	backgrounds = make([]string, 0)
	for _, file := range readDir(dir) {
		fileName := file.Name()
		fullPath := filepath.Join(dir, fileName)
		ext := strings.ToLower(filepath.Ext(fileName))
		if imageExtensions[ext] {
			backgrounds = append(backgrounds, fullPath)
		} else if file.IsDir() {
			folders = append(folders, fullPath)
		}
	}

	if len(backgrounds) == 0 {
		exitApplication(9, fmt.Sprintf("No background images found in directory \"%s\"", dir))
	}
	if len(folders) == 0 {
		exitApplication(9, "Please provide a directory containing images or image folders as collection directory")
	}

	collections = make([]*collection, 0)
	for _, folder := range folders {
		collections = append(collections, &collection{
			Title: filepath.Base(folder),
			files: validatePictureFolder(folder),
		})
	}

	// Create resized pictures and create json
	collectionsJSON := createCollections(outDir, thumbSize, displaySize, backgroundSize, collections, backgrounds)

	// TODO: Copy template
	err = filepath.Walk(templatePath, func(path string, file os.FileInfo, err error) error {
		if err != nil {
			exitApplication(26, fmt.Sprintf("Error copying template file %s: %s", path, err.Error()))
		}

		if path == templatePath {
			// First call with main directory
			return nil
		}

		localPath := path[len(templatePath)+1:]
		targetPath := filepath.Join(outDir, localPath)
		if file.IsDir() {
			mkdir(targetPath)
		} else if localPath == "index.html" {
			// TODO: replace collections
			markerColStart := []byte("/*{{BEGIN:collections*/")
			markerColEnd := []byte("/*END:collections}}*/")

			content, err := ioutil.ReadFile(path)
			if err != nil {
				return err
			}

			start := bytes.Index(content, markerColStart)
			end := bytes.Index(content, markerColEnd)
			if start < 0 || end < 0 || end < start {
				return fmt.Errorf("Cound not find collection marker in template")
			}
			end += len(markerColEnd)

			newContent := make([]byte, 0, len(content)-(end-start)+len(collectionsJSON))
			newContent = append(newContent, content[0:start]...)
			newContent = append(newContent, collectionsJSON...)
			newContent = append(newContent, content[end:]...)

			return ioutil.WriteFile(targetPath, newContent, os.ModePerm)
		} else {
			copyFile(path, targetPath)
		}

		return nil
	})
	if err != nil {
		exitApplication(27, fmt.Sprintf("Error copying template: %s", err.Error()))
	}

	if optimize {
		path := filepath.Join(outDir, "index.html")

		file, err := os.OpenFile(path, os.O_RDWR, os.ModePerm)
		if err != nil {
			exitApplication(28, err.Error())
		}
		doc, err := html.Parse(file)
		if err != nil {
			exitApplication(29, err.Error())
		}

		// Embed CSS
		allStyles := findLinkStyleElements(doc)
		for _, style := range allStyles {
			if style.node.Data == "style" {
				// Embedded style
				if style.node.FirstChild != style.node.LastChild || style.node.FirstChild.Type != html.TextNode {
					exitApplication(30, "Invalid style tag in main template document")
				}
				style.node.FirstChild.Data = minimizeCSS(style.node.FirstChild.Data)
			} else {
				// External link
				stylePath := filepath.Join(outDir, style.path)
				cssdata, err := ioutil.ReadFile(stylePath)
				if err != nil {
					exitApplication(31, err.Error())
				}

				css := &html.Node{
					Type: html.ElementNode,
					Data: "style",
				}
				cssContent := &html.Node{
					Type: html.TextNode,
					Data: minimizeCSS(string(cssdata)),
				}

				css.AppendChild(cssContent)
				style.node.Parent.InsertBefore(css, style.node)
				style.node.Parent.RemoveChild(style.node)

				err = os.Remove(stylePath)
				if err != nil {
					exitApplication(32, err.Error())
				}
				// Remove empty folder, ignore folder if not empty
				os.Remove(filepath.Dir(stylePath))

			}
		}

		// Embed JS
		allScripts := findScriptElements(doc)
		for _, script := range allScripts {
			if script.path == "" {
				// Inline script
				if script.node.FirstChild != script.node.LastChild || script.node.FirstChild.Type != html.TextNode {
					exitApplication(33, "Invalid script tag in main template document")
				}
				script.node.FirstChild.Data = minimizeJS(script.node.FirstChild.Data)
			} else {
				// Embed external script content
				scriptPath := filepath.Join(outDir, script.path)
				scriptdata, err := ioutil.ReadFile(scriptPath)
				if err != nil {
					exitApplication(34, err.Error())
				}

				js := &html.Node{
					Type: html.ElementNode,
					Data: "script",
				}
				jsContent := &html.Node{
					Type: html.TextNode,
					Data: minimizeJS(string(scriptdata)),
				}

				js.AppendChild(jsContent)
				script.node.Parent.InsertBefore(js, script.node)
				script.node.Parent.RemoveChild(script.node)

				err = os.Remove(scriptPath)
				if err != nil {
					exitApplication(35, err.Error())
				}
				// Remove empty folder, ignore folder if not empty
				os.Remove(filepath.Dir(scriptPath))

			}
		}

		// Remove Comments
		deleteCommentNodes(doc)

		file.Truncate(0)
		file.Seek(0, 0)
		err = html.Render(file, doc)
		if err != nil {
			exitApplication(36, err.Error())
		}
	}

}

var rComments = regexp.MustCompile(`(?s)/\*.*?\*/\n{0,}|//.*?\n`)
var rTabs = regexp.MustCompile(`(?s)[\t]+?`)
var rLinebreaks = regexp.MustCompile(`(?s)[\n]+?`)

func minimizeJS(data string) string {
	return removeSpaces(rComments.ReplaceAllString(data, ""))
	/*
		rLinebreaks.ReplaceAllString(
			rTabs.ReplaceAllString(
				rComments.ReplaceAllString(
					data, "",
				), " ",
			), "\n",
		),
	*/
}

func removeSpaces(data string) string {
	sb := strings.Builder{}
	sb.Grow(len(data))

	inString := rune(0)
	var lastB rune
	for _, b := range data {
		if b == '"' || b == '\'' || b == '`' {
			if inString == b {
				// End of current string
				inString = 0
			} else if inString == 0 {
				inString = b
			}
		}

		isSpace := (b == ' ' || b == '\t') && (lastB == ' ' || lastB == '\t' || lastB == '\n')
		isLineBreak := b == '\n' && lastB == '\n'

		if inString != 0 || (!isSpace && !isLineBreak) {
			sb.WriteRune(b)
		}
		lastB = b
	}

	return sb.String()
}

func minimizeCSS(data string) string {
	return rLinebreaks.ReplaceAllString(
		rTabs.ReplaceAllString(
			rComments.ReplaceAllString(
				data, "",
			), "",
		), "",
	)
}

type nodeReference struct {
	node *html.Node
	path string
}

func deleteCommentNodes(node *html.Node) {
	if node.Type == html.CommentNode {
		node.Parent.RemoveChild(node)
	}

	for el := node.FirstChild; el != nil; el = el.NextSibling {
		deleteCommentNodes(el)
	}
}

func findScriptElements(node *html.Node) []nodeReference {
	if node.Type != html.ElementNode && node.Type != html.DocumentNode {
		return []nodeReference{}
	}

	elements := make([]nodeReference, 0)

	scriptPath := ""
	if node.Data == "script" {
		for _, attr := range node.Attr {
			if attr.Key == "src" {
				scriptPath = attr.Val
			}
		}

		nr := nodeReference{
			node: node,
			path: scriptPath,
		}
		elements = append(elements, nr)
	} else {
		for el := node.FirstChild; el != nil; el = el.NextSibling {
			elements = append(elements, findScriptElements(el)...)
		}
	}

	return elements
}

func findLinkStyleElements(node *html.Node) []nodeReference {
	if node.Type != html.ElementNode && node.Type != html.DocumentNode {
		return []nodeReference{}
	}

	elements := make([]nodeReference, 0)

	found := false
	stylePath := ""
	if node.Data == "link" {
		for _, attr := range node.Attr {
			if attr.Key == "rel" && attr.Val == "stylesheet" {
				found = true
			}
			if attr.Key == "href" {
				stylePath = attr.Val
			}
		}
	} else if node.Data == "style" {
		found = true
	}

	if found {
		nr := nodeReference{
			node: node,
			path: stylePath,
		}
		elements = append(elements, nr)
	} else {
		for el := node.FirstChild; el != nil; el = el.NextSibling {
			elements = append(elements, findLinkStyleElements(el)...)
		}
	}

	return elements
}

func filterDirs(entries []os.FileInfo) []string {
	dirs := make([]string, 0, len(entries))

	for _, entry := range entries {
		if entry.IsDir() {
			dirs = append(dirs, entry.Name())
		}
	}

	return dirs
}

func createCollections(outDir string, ts, ds, bs size, collections []*collection, backgrounds []string) []byte {
	bgDir := "b"
	bgDirFull := filepath.Join(outDir, bgDir)
	mkdir(bgDirFull)

	newBackgrounds := make([]string, len(backgrounds))
	for i, path := range backgrounds {
		bgName := strconv.FormatInt(int64(i), 36) + ".jpg"
		resizePicture(bgName, bs.width, bs.height, path, bgDirFull)
		newBackgrounds[i] = filepath.Join(bgDir, bgName)
	}

	for n, col := range collections {
		col.Backgrounds = newBackgrounds

		colDir := "c" + strconv.FormatInt(int64(n), 36)
		colDirFull := filepath.Join(outDir, colDir)
		mkdir(colDirFull)

		col.Pictures = make([]pictureEntry, len(col.files))
		for i, original := range col.files {
			targetName := strconv.FormatInt(int64(i), 36)
			targetCopy := filepath.Join(colDir, targetName+".jpg")

			copyFile(original, filepath.Join(outDir, targetCopy))
			resizePicture(targetName+"-p.jpg", ds.width, ds.height, original, colDirFull)
			resizePicture(targetName+"-t.jpg", ts.width, ts.height, original, colDirFull)

			col.Pictures[i].Fullsize = targetCopy
			col.Pictures[i].Picture = filepath.Join(colDir, targetName+"-p.jpg")
			col.Pictures[i].Thumb = filepath.Join(colDir, targetName+"-t.jpg")
		}
	}

	return createJSON(collections)
}

func createJSON(data interface{}) []byte {
	jsonData, err := json.Marshal(data)
	if err != nil {
		exitApplication(23, err.Error())
	}
	return jsonData
}

func copyPicture(from, toDir string) string {
	in, err := os.Stat(from)
	if err != nil {
		exitApplication(14, err.Error())
	}
	dir, err := os.Stat(toDir)
	if err != nil || !dir.IsDir() {
		exitApplication(15, fmt.Sprintf("Could not access target dir %s\n", toDir))
	}

	to := filepath.Join(toDir, in.Name())
	copyFile(from, to)

	return to
}

func copyFile(from, to string) {
	read, err := os.Open(from)
	if err != nil {
		exitApplication(16, fmt.Sprintf("Error reading file \"%s\": %s\n", from, err.Error()))
	}
	defer read.Close()

	write, err := os.Create(to)
	if err != nil {
		exitApplication(17, fmt.Sprintf("Error creating file \"%s\": %s\n", to, err.Error()))
	}
	defer write.Close()

	_, err = io.Copy(write, read)
	if err != nil {
		exitApplication(18, fmt.Sprintf("Error copying to file \"%s\": %s\n", to, err.Error()))
	}
}

func resizePicture(newName string, targetX, targetY int, originalPath string, targetDir string) string {
	file, err := os.Open(originalPath)
	if err != nil {
		exitApplication(19, err.Error())
	}
	defer file.Close()

	img, _, err := image.Decode(file)
	if err != nil {
		exitApplication(20, err.Error())
	}

	size := img.Bounds().Size()

	var newImg image.Image
	if size.X > targetX || size.Y > targetY {
		// Picture is larger than the background size, resize
		newImg = resize.Thumbnail(uint(targetX), uint(targetY), img, resize.Lanczos2)
	} else {
		// Picture is smaller, just copy the picture
		newImg = img
	}

	newPath := filepath.Join(targetDir, newName)
	newFile, err := os.OpenFile(newPath, os.O_CREATE|os.O_WRONLY, os.ModePerm)
	if err != nil {
		exitApplication(21, err.Error())
	}

	err = jpeg.Encode(newFile, newImg, &jpeg.Options{Quality: 70})
	if err != nil {
		exitApplication(22, err.Error())
	}

	return newPath
}

///////////////////////////////////////////////////////////////////////////////////7

func mkdir(dir string) {
	err := os.MkdirAll(dir, os.ModePerm)
	if err != nil {
		exitApplication(10, fmt.Sprintf("Could not create directory \"%s\"", dir))
	}
}

func readDir(dir string) []os.FileInfo {
	s, err := os.Stat(dir)
	if err != nil {
		exitApplication(5, fmt.Sprintf("Error opening folder \"%s\": %s", dir, err.Error()))
	}
	if !s.IsDir() {
		exitApplication(6, fmt.Sprintf("Error opening folder \"%s\": Not a directory", dir))
	}

	files, err := ioutil.ReadDir(dir)
	if err != nil {
		exitApplication(7, fmt.Sprintf("Error opening folder \"%s\": %s", dir, err.Error()))
	}

	return files
}

func validatePictureFolder(dir string) []string {
	files := readDir(dir)
	pictures := make([]string, 0)
	for _, file := range files {
		fileName := file.Name()
		ext := strings.ToLower(filepath.Ext(fileName))
		if !file.IsDir() && imageExtensions[ext] {
			pictures = append(pictures, filepath.Join(dir, fileName))
		} else if !file.IsDir() {
			log.Printf("Ignored unknown image file: %s", fileName)
		} else {
			// Ignore subfolders
		}
	}

	if len(pictures) == 0 {
		exitApplication(8, fmt.Sprintf("Picture-folder does not contain pictures: \"%s\"", dir))
	}

	return pictures
}

func validateOutputDir(dir string) {
	if dir == "" {
		exitApplication(24, "Please provide an output directory using option \"-output\"")
	}

	f, err := os.Stat(dir)
	if os.IsNotExist(err) {
		// Great, let's create it
		mkdir(dir)
	} else if !f.IsDir() {
		exitApplication(11, fmt.Sprintf("Not a directory: \"%s\"", dir))
	}

	contents, err := ioutil.ReadDir(dir)
	if err != nil {
		exitApplication(12, fmt.Sprintf("Error reading output directory \"%s\": %s", dir, err.Error()))
	} else if len(contents) != 0 {
		exitApplication(13, fmt.Sprintf("Output directory is not empty: \"%s\"", dir))
	}
}

func parseSize(sizeString string) (size, error) {
	parts := strings.Split(sizeString, "x")

	if len(parts) != 2 {
		return size{}, fmt.Errorf("Invalid resolution string: \"%s\"", sizeString)
	}

	width, err := strconv.ParseInt(parts[0], 10, 0)
	if err != nil || width < 1 {
		return size{}, fmt.Errorf("Invalid resolution width: \"%s\"", parts[0])
	}

	height, err := strconv.ParseInt(parts[1], 10, 0)
	if err != nil || height < 1 {
		return size{}, fmt.Errorf("Invalid resolution height: \"%s\"", parts[1])
	}

	return size{width: int(width), height: int(height)}, nil
}

func showHelp() {
	fmt.Fprint(flag.CommandLine.Output(), "\n\n")
	fmt.Fprint(flag.CommandLine.Output(), "Usage:\n")
	fmt.Fprint(flag.CommandLine.Output(), "    generate-gallery generate-gallery -output foldername -template path/to/template path/to/collection(s) path/to/collection(s)\n")
	fmt.Fprint(flag.CommandLine.Output(), "\n")
	fmt.Fprint(flag.CommandLine.Output(), "The path must be a folder directly containing the background images and a folder containing images for each collection.\n")
	fmt.Fprint(flag.CommandLine.Output(), "\n")
	flag.PrintDefaults()
	os.Exit(0)
}

func exitApplication(code int, message string) {
	fmt.Fprint(flag.CommandLine.Output(), "\n")
	fmt.Fprint(flag.CommandLine.Output(), "ERROR: ")
	fmt.Fprint(flag.CommandLine.Output(), message)
	fmt.Fprint(flag.CommandLine.Output(), "\n")
	os.Exit(code)
}
